use std::{collections::HashMap, sync::Arc, time::Duration};

use bollard::{
    query_parameters::{
        CreateContainerOptionsBuilder, InspectContainerOptions, RemoveContainerOptionsBuilder,
        StartContainerOptions,
    },
    secret::{
        ContainerCreateBody, EndpointSettings, HostConfig, NetworkCreateRequest, NetworkingConfig,
        PortBinding,
    },
};
use chrono::NaiveDateTime;
use deployer_common::challenge::{Container, DeployableContext, DeploymentStrategy, ExposeType};
use eyre::eyre;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{State, api::ChallengeDeploymentRow, config::CaddyKeychain};

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
            data: self.data.map(|d| {
                d.into_iter()
                    .map(|(k, v)| {
                        (
                            k,
                            DeploymentDataS {
                                container_id: "redacted-xxxxx".to_owned(),
                                ..v
                            },
                        )
                    })
                    .collect()
            }),
            ..self
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeploymentDataS {
    pub container_id: String,
    pub ports: HashMap<u16, HostMapping>,
}

pub type DeploymentData = HashMap<String, DeploymentDataS>;

// keep this in sync with ExposeType
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum HostMapping {
    Tcp { port: u16, base: String },
    // subdomain name
    Http { subdomain: String, base: String },
}

fn calculate_container_name(
    chall_id: &str,
    strategy: DeploymentStrategy,
    ct: &str,
    team_id: Option<i32>,
) -> String {
    match strategy {
        DeploymentStrategy::Static => format!("{}-container-{}", chall_id, ct),
        DeploymentStrategy::Instanced => {
            format!("{}-team-{}-container-{}", chall_id, team_id.unwrap(), ct)
        }
    }
}

fn calculate_network_name(
    chall_id: &str,
    strategy: DeploymentStrategy,
    team_id: Option<i32>,
) -> String {
    match strategy {
        DeploymentStrategy::Static => format!("{}-network", chall_id),
        DeploymentStrategy::Instanced => format!("{}-team-{}-network", chall_id, team_id.unwrap()),
    }
}

fn get_unused_port() -> u16 {
    loop {
        if let Ok(l) = std::net::TcpListener::bind(("0.0.0.0", 0)) {
            return l.local_addr().unwrap().port();
        }
    }
}

fn calculate_subdomain(chall_id: &str, pub_team_id: Option<&str>, port: u16) -> String {
    let h = {
        use sha2::Digest;
        use std::io::Write;

        let mut hasher = sha2::Sha256::new();
        write!(
            hasher,
            "{}/{}/{}",
            chall_id,
            pub_team_id.unwrap_or(""),
            port
        )
        .unwrap();
        hasher.finalize()
    };
    // take first 40 bits (40 mod 5 = 0)
    let num = &h[..5];
    let end = fast32::base32::CROCKFORD_LOWER.encode(num);
    format!("{}-{}", chall_id, end)
}

#[derive(Debug, Clone)]
struct DockerGuard {
    ctx: Arc<DeployableContext>,
    containers: Vec<String>,
    networks: Vec<String>,
    committed: bool,
    dropping: bool,
}

impl DockerGuard {
    pub fn new(ctx: Arc<DeployableContext>) -> Self {
        Self {
            ctx,
            containers: vec![],
            networks: vec![],
            committed: false,
            dropping: false,
        }
    }

    pub fn container(&mut self, s: &str) {
        self.containers.push(s.to_owned());
    }

    pub fn network(&mut self, n: &str) {
        self.networks.push(n.to_owned());
    }

    pub fn commit(&mut self) {
        self.committed = true;
    }

    pub async fn adrop(self) {
        if self.committed {
            return;
        }

        for c in self.containers.iter().rev() {
            self.ctx
                .docker
                .remove_container(
                    c,
                    Some(
                        RemoveContainerOptionsBuilder::new()
                            .v(true)
                            .force(true)
                            .build(),
                    ),
                )
                .await
                .ok();
        }

        for n in self.networks.iter().rev() {
            self.ctx.docker.remove_network(n).await.ok();
        }
    }
}

impl Drop for DockerGuard {
    fn drop(&mut self) {
        if self.dropping {
            return;
        }
        self.dropping = true;
        let self2 = self.clone();
        tokio::spawn(async move {
            self2.adrop().await;
        });
    }
}

#[derive(Debug, Clone)]
struct CaddyGuard {
    client: Arc<reqwest::Client>,
    kc: CaddyKeychain,
    routes: Vec<String>,
    committed: bool,
    dropping: bool,
}

impl CaddyGuard {
    pub fn new(client: Arc<reqwest::Client>, kc: CaddyKeychain) -> Self {
        Self {
            client,
            kc,
            routes: vec![],
            committed: true,
            dropping: false,
        }
    }

    pub fn route(&mut self, r: &str) {
        self.routes.push(r.to_owned());
    }

    pub fn commit(&mut self) {
        self.committed = true;
    }

    pub async fn adrop(self) {
        if self.committed {
            return;
        }

        for r in self.routes.iter().rev() {
            self.client
                .post(self.kc.prep_url("/dynamic-router/delete"))
                .json(&json!({
                    "host": r,
                }))
                .send()
                .await
                .ok();
        }
    }
}

impl Drop for CaddyGuard {
    fn drop(&mut self) {
        if self.dropping {
            return;
        }
        self.dropping = true;
        let self2 = self.clone();
        tokio::spawn(async move {
            self2.adrop().await;
        });
    }
}

pub async fn deploy_challenge(
    state: State,
    tx: &mut sqlx::PgTransaction<'_>,
    chall: ChallengeDeployment,
) -> eyre::Result<()> {
    // 1. find the public id of the challenge ("slug")
    // TODO(aiden): replace with query_scalar!
    let public_chall_partial = sqlx::query!(
        "SELECT public_id FROM challenges WHERE id = $1",
        chall.challenge_id
    )
    .fetch_one(&mut **tx)
    .await?;

    // 1.1 get public team id
    let public_team_id = if let Some(tid) = chall.team_id {
        Some(
            sqlx::query!("SELECT public_id FROM teams WHERE id = $1", tid,)
                .fetch_one(&mut **tx)
                .await?
                .public_id,
        )
    } else {
        None
    };

    // 2. find the challenge data for that slug
    let chall_data = {
        let rg = state.challenge_data.read().await;
        rg.get(&public_chall_partial.public_id).map(Clone::clone)
    }
    .ok_or_else(|| {
        eyre!(
            "failed to get challenge data for {}",
            public_chall_partial.public_id
        )
    })?;

    // 3. ensure there is a container on it
    let Some(chall_containers) = &chall_data.container else {
        return Err(eyre!("challenge {} does not have container", chall_data.id));
    };

    // 4. connect to the appropriate docker socket
    let host_keychain =
        &state.config.host_keychains[chall_data.host.as_deref().unwrap_or("default")];
    let ctx: Arc<DeployableContext> = Arc::new(host_keychain.docker.clone().try_into()?);

    // think these steps can be repeated for each container (perhaps create a network?)
    let mut _docker_guard = DockerGuard::new(ctx.clone());
    let caddy_client = Arc::new(host_keychain.caddy.as_client()?);
    let mut _caddy_guard = CaddyGuard::new(caddy_client.clone(), host_keychain.caddy.clone());

    let mut deploy_data = HashMap::new();

    /* TODO: create the network */
    let network_name = calculate_network_name(&chall_data.id, chall_data.strategy, chall.team_id);
    ctx.docker.remove_network(&network_name).await.ok();
    ctx.docker
        .create_network(NetworkCreateRequest {
            name: network_name.clone(),
            ..Default::default()
        })
        .await?;
    _docker_guard.network(&network_name);

    // 5.2. pull the container image
    chall_data.pull(&ctx).await?;

    debug!("pulled image, creating...");

    for (ct, chall_container) in chall_containers {
        // 4. calculate the container name
        let container_name =
            calculate_container_name(&chall_data.id, chall_data.strategy, ct, chall.team_id);

        debug!("calculated container name: {}", container_name);

        // 5. determine host mappings
        let mut mappings = HashMap::new();
        if let Some(expose) = &chall_container.expose {
            for (&p, &t) in expose {
                match t {
                    ExposeType::Tcp => {
                        mappings.insert(
                            p,
                            HostMapping::Tcp {
                                port: get_unused_port(),
                                base: host_keychain.caddy.base.clone(),
                            },
                        );
                    }
                    ExposeType::Http => {
                        mappings.insert(
                            p,
                            HostMapping::Http {
                                subdomain: calculate_subdomain(
                                    &chall_data.id,
                                    public_team_id.as_deref(),
                                    p,
                                ),
                                base: host_keychain.caddy.base.clone(),
                            },
                        );
                    }
                }
            }
        }

        debug!("calculated mappings: {:#?}", mappings);

        // 6. create container with tcp mappings
        // TODO: maybe also want to expose http ports if we use networks later
        ctx.docker
            .remove_container(
                &container_name,
                Some(
                    RemoveContainerOptionsBuilder::new()
                        .v(true)
                        .force(true)
                        .build(),
                ),
            )
            .await
            .ok();
        ctx.docker
            .create_container(
                Some(
                    CreateContainerOptionsBuilder::new()
                        .name(&container_name)
                        .build(),
                ),
                ContainerCreateBody {
                    env: chall_container.env.as_ref().map(|h| {
                        h.iter()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .collect::<Vec<_>>()
                    }),
                    /* todo: resource limits */
                    image: Some(chall_data.image_id(&ctx, ct)),
                    networking_config: Some(NetworkingConfig {
                        endpoints_config: Some({
                            let mut h = HashMap::new();
                            h.insert(
                                network_name.clone(),
                                EndpointSettings {
                                    aliases: Some(vec![ct.clone()]),
                                    ..Default::default()
                                },
                            );
                            h
                        }),
                    }),
                    exposed_ports: Some(
                        mappings
                            .iter()
                            .filter(|(_, v)| matches!(v, HostMapping::Tcp { .. }))
                            .map(|(k, _)| (format!("{}/tcp", k), Default::default()))
                            .collect::<HashMap<_, _>>(),
                    ),
                    host_config: Some(HostConfig {
                        nano_cpus: chall_container.limits.cpu, // nanocpus (10 ^ -9 cpus)
                        memory: chall_container.limits.mem,    // bytes
                        port_bindings: Some(
                            mappings
                                .iter()
                                .filter_map(|(k, v)| match v {
                                    HostMapping::Tcp { port: p, .. } => Some((*k, *p)),
                                    _ => None,
                                })
                                .map(|(p1, p2)| {
                                    (
                                        format!("{}/tcp", p1),
                                        Some(vec![PortBinding {
                                            host_ip: Some("0.0.0.0".to_owned()),
                                            host_port: Some(format!("{}", p2)),
                                        }]),
                                    )
                                })
                                .collect::<HashMap<_, _>>(),
                        ),
                        privileged: chall_container.privileged.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await?;
        _docker_guard.container(&container_name);

        debug!("starting container");

        // 7. start container
        ctx.docker
            .start_container(&container_name, None::<StartContainerOptions>)
            .await?;

        // 8. inspect container to get its ip
        let container_ip = {
            let container_inspected = ctx
                .docker
                .inspect_container(&container_name, None::<InspectContainerOptions>)
                .await?;
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

        for (p, map) in &mappings {
            if let HostMapping::Http { subdomain, .. } = &map {
                let host = format!("{}.{}", subdomain, host_keychain.caddy.base);
                caddy_client
                    .post(host_keychain.caddy.prep_url("/dynamic-router/delete"))
                    .json(&json!({
                        "host": host,
                    }))
                    .send()
                    .await?;
                caddy_client
                    .post(host_keychain.caddy.prep_url("/dynamic-router/add"))
                    .json(&json!({
                        "host": host,
                        "upstream": format!("{}:{}", container_ip, p),
                    }))
                    .send()
                    .await?;
                _caddy_guard.route(&host);
            }
        }

        deploy_data.insert(
            ct,
            DeploymentDataS {
                container_id: container_name,
                ports: mappings,
            },
        );
    }

    // 10. determine new expiration time if necessary
    let new_expiration_time = match chall_data.strategy {
        DeploymentStrategy::Static => None,
        DeploymentStrategy::Instanced => {
            Some(chrono::Utc::now().naive_utc() + Duration::from_secs(60 * 10))
        }
    };

    // 11. update the db
    sqlx::query!(
        "UPDATE challenge_deployments SET deployed = TRUE, data = $2, expired_at = $3 WHERE id = $1",
        chall.id,
        Some(serde_json::to_value(deploy_data)?),
        new_expiration_time,
    )
        .execute(&mut **tx)
        .await?;

    // 12. spawn a task to destroy the challenge after the expiration duration (todo)
    if let Some(expiration_time) = new_expiration_time {
        let dur = (expiration_time - chrono::Utc::now().naive_utc())
            .to_std()
            .unwrap();
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

    _docker_guard.commit();
    _caddy_guard.commit();
    Ok(())
}

pub async fn deploy_challenge_task(state: State, chall: ChallengeDeployment) {
    let mut tx = state.db.begin().await.unwrap();
    if let Err(e) = deploy_challenge(state, &mut tx, chall.clone()).await {
        error!("Failed to deploy challenge {:?}: {:?}", chall, e);
        sqlx::query!("DELETE FROM challenge_deployments WHERE id = $1", chall.id,)
            .execute(&mut *tx)
            // idk
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
}

pub async fn destroy_challenge(
    state: State,
    tx: &mut sqlx::PgTransaction<'_>,
    chall: ChallengeDeployment,
) -> eyre::Result<()> {
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
    let chall_data = match {
        let rg = state.challenge_data.read().await;
        rg.get(&public_chall_partial.public_id).map(Clone::clone)
    } {
        Some(x) => x,
        _ => return Ok(()),
    };

    // 3. ensure there is a container on it
    let Some(chall_containers) = &chall_data.container else {
        return Ok(());
    };

    // 4. connect to the appropriate docker socket
    let host_keychain =
        &state.config.host_keychains[chall_data.host.as_deref().unwrap_or("default")];
    let ctx: DeployableContext = host_keychain.docker.clone().try_into()?;
    let caddy_client = host_keychain.caddy.as_client()?;

    // think these steps can be repeated for each container (perhaps create a network?)
    for (ct, _chall_container) in chall_containers {
        // 4. calculate the container name
        let container_name =
            calculate_container_name(&chall_data.id, chall_data.strategy, ct, chall.team_id);

        debug!("calculated container name: {}", container_name);

        // ok now delete the caddy stuff
        if let Some(dd) = deploy_data.get(ct) {
            for (_p, map) in &dd.ports {
                if let HostMapping::Http { subdomain, .. } = &map {
                    let host = format!("{}.{}", subdomain, host_keychain.caddy.base);
                    caddy_client
                        .post(host_keychain.caddy.prep_url("/dynamic-router/delete"))
                        .json(&json!({
                            "host": host,
                        }))
                        .send()
                        .await?;
                }
            }
        }

        // kill the container
        ctx.docker
            .remove_container(
                &container_name,
                Some(
                    RemoveContainerOptionsBuilder::new()
                        .v(true)
                        .force(true)
                        .build(),
                ),
            )
            .await
            .ok();
    }

    /* TODO: delete network */
    let network_name = calculate_network_name(&chall_data.id, chall_data.strategy, chall.team_id);
    ctx.docker.remove_network(&network_name).await.ok();

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
