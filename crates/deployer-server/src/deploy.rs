use std::{collections::HashMap, time::Duration};

use bollard::{query_parameters::{CreateContainerOptionsBuilder, InspectContainerOptions, RemoveContainerOptionsBuilder, StartContainerOptions}, secret::{ContainerCreateBody, HostConfig, PortBinding}};
use deployer_common::challenge::{Container, ContainerStrategy, DeployableContext, ExposeType};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use eyre::eyre;

use crate::{api::ChallengeDeploymentRow, State};

/* db models (sorta) */
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChallengeDeployment {
    #[serde(skip_serializing)]
    pub id: i32,
    #[serde(rename(serialize = "id"))]
    pub public_id: String,
    #[serde(skip_serializing)]
    pub team_id: Option<i32>,
    #[serde(skip_serializing)]
    pub challenge_id: i32,
    pub deployed: bool,
    pub data: Option<DeploymentData>,
    pub created_at: NaiveDateTime,
    pub expired_at: Option<NaiveDateTime>,
    pub destroyed_at: Option<NaiveDateTime>,
}

impl ChallengeDeployment {
    // TODO(ani): hacky solution
    pub fn sanitize(self) -> Self {
        Self {
            data: self.data.map(|d| DeploymentData {
                container_id: "redacted-xxxxx".to_owned(),
                ..d
            }),
            ..self
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeploymentData {
    pub container_id: String,
    pub ports: HashMap<u16, HostMapping>,
}

// keep this in sync with ExposeType
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum HostMapping {
    Tcp {
        port: u16,
    },
    // subdomain name
    Http {
        subdomain: String,
        base: String,
    },
}

fn calculate_container_name(
    chall_id: &str,
    container: &Container,
    team_id: Option<i32>,
) -> String {
    match container.strategy {
        ContainerStrategy::Static => format!("{}-container", chall_id),
        ContainerStrategy::Instanced => format!("{}-team-{}-container", chall_id, team_id.unwrap()),
    }
}

fn get_unused_port() -> u16 {
    loop {
        if let Ok(l) = std::net::TcpListener::bind(("0.0.0.0", 0)) {
            return l.local_addr().unwrap().port();
        }
    }
}

fn calculate_subdomain(
    chall_id: &str,
    pub_team_id: Option<&str>,
    port: u16,
) -> String {
    let h = {
        use std::io::Write;
        use sha2::Digest;

        let mut hasher = sha2::Sha256::new();
        write!(hasher, "{}/{}/{}", chall_id, pub_team_id.unwrap_or(""), port).unwrap();
        hasher.finalize()
    };
    // take first 40 bits (40 mod 5 = 0)
    let num = &h[..5];
    let end = fast32::base32::CROCKFORD_LOWER.encode(num);
    format!("{}-{}", chall_id, end)
}

pub async fn deploy_challenge(state: State, tx: &mut sqlx::PgTransaction<'_>, chall: ChallengeDeployment) -> eyre::Result<()> {
    // 1. find the public id of the challenge ("slug")
    let public_chall_partial = sqlx::query!(
        "SELECT public_id FROM challenges WHERE id = $1",
        chall.challenge_id
    )
        .fetch_one(&mut **tx)
        .await?;

    // 1.1 get public team id
    let public_team_id = if let Some(tid) = chall.team_id {
        Some(sqlx::query!(
            "SELECT public_id FROM teams WHERE id = $1",
            tid,
        )
            .fetch_one(&mut **tx)
            .await?.public_id)
    } else { None };


    // 2. find the challenge data for that slug
    let chall_data = state.challenge_data.get(&public_chall_partial.public_id)
        .ok_or_else(|| eyre!("failed to get challenge data for {}", public_chall_partial.public_id))?;

    // 3. ensure there is a container on it
    let Some(chall_container) = &chall_data.container else {
        return Err(eyre!("challenge {} does not have container", chall_data.id));
    };

    // 4. connect to the appropriate docker socket
    // TODO: read a host keychain field from the chall data later
    let host_keychain = &state.config.host_keychains["default"];
    let ctx: DeployableContext = host_keychain.docker.clone().try_into()?;

    // think these steps can be repeated for each container (perhaps create a network?)

    // 4. calculate the container name
    let container_name = calculate_container_name(&chall_data.id, &chall_container, chall.team_id);

    debug!("calculated container name: {}", container_name);

    // 5. determine host mappings
    let mut mappings = HashMap::new();
    if let Some(expose) = &chall_container.expose {
        for (&p, &t) in expose {
            match t {
                ExposeType::Tcp => {
                    mappings.insert(p, HostMapping::Tcp {
                        port: get_unused_port(),
                    });
                }
                ExposeType::Http => {
                    mappings.insert(p, HostMapping::Http {
                        subdomain: calculate_subdomain(&chall_data.id, public_team_id.as_deref(), p),
                        base: host_keychain.caddy.base.clone(),
                    });
                }
            }
        }
    }

    debug!("calculated mappings: {:#?}", mappings);

    // 5.2. pull the container image
    chall_data.pull(&ctx).await?;

    debug!("pulled image, creating...");

    // 6. create container with tcp mappings
    // TODO: maybe also want to expose http ports if we use networks later
    ctx.docker.create_container(
        Some(CreateContainerOptionsBuilder::new()
            .name(&container_name)
            .build()),
        ContainerCreateBody {
            /* todo: env */
            /* todo: resource limits */
            image: Some(chall_data.image_id(&ctx)),
            exposed_ports: Some(mappings
                .iter()
                .filter(|(_, v)| matches!(v, HostMapping::Tcp { .. }))
                .map(|(k, _)| (format!("{}/tcp", k), Default::default()))
                .collect::<HashMap<_, _>>()),
            host_config: Some(HostConfig {
                port_bindings: Some(mappings
                    .iter()
                    .filter_map(|(k, v)| match v {
                        HostMapping::Tcp { port: p } => Some((*k, *p)),
                        _ => None,
                    })
                    .map(|(p1, p2)| (format!("{}/tcp", p1), Some(vec![PortBinding {
                        host_ip: Some("0.0.0.0".to_owned()),
                        host_port: Some(format!("{}", p2)),
                    }])))
                    .collect::<HashMap<_, _>>()
                ),
                privileged: chall_container.privileged.clone(),
                ..Default::default()
            }),
            ..Default::default()
        },
    ).await?;

    debug!("starting container");

    // 7. start container
    ctx.docker.start_container(&container_name, None::<StartContainerOptions>).await?;

    // 8. inspect container to get its ip
    let container_ip = {
        let container_inspected = ctx.docker.inspect_container(&container_name, None::<InspectContainerOptions>).await?;
        debug!("got inspected: {:?}", container_inspected);
        container_inspected
            .network_settings
            .ok_or_else(|| eyre!("Container has no network settings"))?
            .networks
            .ok_or_else(|| eyre!("Container has no networks"))?
            .iter()
            .next()
            .ok_or_else(|| eyre!("Container has no networks"))?
            .1
            .ip_address
            .clone()
            .ok_or_else(|| eyre!("Container has no IP address"))?
    };

    debug!("creating caddy client");

    // 9. ??? update caddy or something somehow
    // FIXME(ani): guarding since caddy client thing doesn't work rn
    if mappings.iter().any(|(_, v)| matches!(v, HostMapping::Http { .. })) {
        let caddy_client = host_keychain.caddy.as_client()?;

        for (p, map) in &mappings {
            if let HostMapping::Http { subdomain, .. } = &map {
                let caddy_id = format!("proxy-{}", subdomain);
                caddy_client
                    .delete(host_keychain.caddy.prep_url(&format!("/id/{}", caddy_id)))
                    .send()
                    .await?;
                caddy_client
                    .put(host_keychain.caddy.prep_url("/id/default-server/routes/0"))
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&serde_json::json!({
                        "@id": caddy_id,
                        "match": [{
                            "host": [format!("{}.{}", subdomain, host_keychain.caddy.base)],
                        }],
                        "handle": [{
                            "handler": "reverse_proxy",
                            "upstreams": [{
                                "dial": format!("{}:{}", container_ip, p),
                            }]
                        }],
                    }))?)
                    .send()
                    .await?;
            }
        }
    }

    // 10. determine new expiration time if necessary
    let new_expiration_time = match chall_container.strategy {
        ContainerStrategy::Static => None,
        ContainerStrategy::Instanced => Some(chrono::Utc::now().naive_utc() + Duration::from_secs(60 * 10)),
    };
    
    // 11. update the db
    sqlx::query!(
        "UPDATE challenge_deployments SET deployed = TRUE, data = $2, expired_at = $3 WHERE id = $1",
        chall.id,
        Some(serde_json::to_value(DeploymentData {
            container_id: container_name,
            ports: mappings,
        })?),
        new_expiration_time,
    )
        .execute(&mut **tx)
        .await?;

    // 12. spawn a task to destroy the challenge after the expiration duration (todo)
    if let Some(expiration_time) = new_expiration_time {
        let dur = (expiration_time - chrono::Utc::now().naive_utc()).to_std().unwrap();
        let state2 = state.clone();
        let chall2 = sqlx::query_as!(
            ChallengeDeploymentRow,
            "SELECT * FROM challenge_deployments WHERE id = $1",
            chall.id,
        )
            .fetch_one(&mut **tx)
            .await?
            .try_into()?;
        tokio::spawn(async move {
            tokio::time::sleep(dur).await;
            destroy_challenge_task(state2, chall2).await;
        });
    }

    Ok(())
}

pub async fn deploy_challenge_task(state: State, chall: ChallengeDeployment) {
    let mut tx = state.db.begin().await.unwrap();
    if let Err(e) = deploy_challenge(state, &mut tx, chall.clone()).await {
        error!("Failed to deploy challenge {:?}: {:?}", chall, e);
        sqlx::query!(
            "DELETE FROM challenge_deployments WHERE id = $1",
            chall.id,
        )
            .execute(&mut *tx)
            // idk
            .await.unwrap();
    }
    tx.commit().await.unwrap();
}

pub async fn destroy_challenge(state: State, tx: &mut sqlx::PgTransaction<'_>, chall: ChallengeDeployment) -> eyre::Result<()> {
    if chall.destroyed_at.is_some() {
        return Ok(());
    }

    // this will get dropped if the destroy fails
    sqlx::query!(
        "UPDATE challenge_deployments SET data = NULL, destroyed_at = NOW() WHERE id = $1",
        chall.id,
    )
        .execute(&mut **tx)
        .await?;

    // ???
    if !chall.deployed {
        return Ok(());
    }

    // ???
    let Some(deploy_data) = &chall.data else {
        return Ok(());
    };

    // grafted from deploy (TODO: dedupe this somehow)

    // 1. find the public id of the challenge ("slug")
    let public_chall_partial = sqlx::query!(
        "SELECT public_id FROM challenges WHERE id = $1",
        chall.challenge_id
    )
        .fetch_one(&mut **tx)
        .await?;

    // 2. find the challenge data for that slug
    let chall_data = state.challenge_data.get(&public_chall_partial.public_id)
        .ok_or_else(|| eyre!("failed to get challenge data for {}", public_chall_partial.public_id))?;

    // 3. ensure there is a container on it
    let Some(chall_container) = &chall_data.container else {
        return Err(eyre!("challenge {} does not have container", chall_data.id));
    };

    // 4. connect to the appropriate docker socket
    // TODO: read a host keychain field from the chall data later
    let host_keychain = &state.config.host_keychains["default"];
    let ctx: DeployableContext = host_keychain.docker.clone().try_into()?;

    // think these steps can be repeated for each container (perhaps create a network?)

    // 4. calculate the container name
    let container_name = calculate_container_name(&chall_data.id, &chall_container, chall.team_id);

    debug!("calculated container name: {}", container_name);

    // ok now delete the caddy stuff
    // FIXME(ani): guarding since caddy client thing doesn't work rn
    if deploy_data.ports.iter().any(|(_, v)| matches!(v, HostMapping::Http { .. })) {
        let caddy_client = host_keychain.caddy.as_client()?;

        for (_p, map) in &deploy_data.ports {
            if let HostMapping::Http { subdomain, .. } = &map {
                let caddy_id = format!("proxy-{}", subdomain);
                caddy_client
                    .delete(host_keychain.caddy.prep_url(&format!("/id/{}", caddy_id)))
                    .send()
                    .await?;
            }
        }
    }

    // kill the container
    ctx.docker.remove_container(&container_name, Some(RemoveContainerOptionsBuilder::new()
        .v(true)
        .force(true)
        .build())).await?;

    // done... how nice

    Ok(())
}

pub async fn destroy_challenge_task(state: State, chall: ChallengeDeployment) {
    let mut tx = state.db.begin().await.unwrap();
    if let Err(e) = destroy_challenge(state, &mut tx, chall.clone()).await {
        error!("Failed to destroy challenge {:?}: {:?}", chall, e);
        // don't commit the tx
    } else {
        tx.commit().await.unwrap();
    }
}
