use bollard::query_parameters::CreateImageOptionsBuilder;
use eyre::{Context, Result, eyre};
use flate2::{Compression, write::GzEncoder};
use google_cloud_storage::{
    client::Client,
    http::objects::upload::{Media, UploadObjectRequest, UploadType},
    sign::SignedURLOptions,
};
use log::info;
use serde_with::{DisplayFromStr, serde_as};
use std::{
    collections::HashMap,
    fs::{self, File as StdFile},
    io::Read,
    path::PathBuf,
    time::Duration,
};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tempdir::TempDir;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Challenge {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub flag: Flag,
    pub visible: Option<bool>,
    // TODO in the future we should make this one, each challenge can belong to multiple groups
    // and if a group is marked as a build group it will be excluded by default.
    pub group: Option<String>,
    pub build_group: Option<String>,
    pub category: String,
    pub points: PointRange,
    pub provide: Option<Vec<Attachment>>,
    pub container: Option<HashMap<String, Container>>,
    #[serde(default = "default_strategy")]
    pub strategy: DeploymentStrategy,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub bump_seed: u64,
    pub host: Option<String>,
}

fn is_zero(x: &u64) -> bool {
    *x == 0
}

fn default_strategy() -> DeploymentStrategy {
    DeploymentStrategy::Static
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PointRange {
    pub min: i32,
    pub max: i32,
}

impl Challenge {
    pub fn image_id(&self, ctx: &DeployableContext, ct: &str) -> String {
        if ct == "default" {
            format!("{}/{}{}", ctx.repo, ctx.image_prefix, self.id)
        } else {
            format!("{}/{}{}-{}", ctx.repo, ctx.image_prefix, self.id, ct)
        }
    }

    pub async fn push_ct(&self, ctx: &DeployableContext, ct: &str) -> Result<()> {
        // TODO: support credentials
        let mut push = ctx.docker.push_image(
            &self.image_id(ctx, ct),
            None::<bollard::query_parameters::PushImageOptions>,
            ctx.docker_credentials.clone(),
        );

        while let Some(push_step) = push.next().await {
            let push_step = push_step.context("Docker image push error")?;
            info!("{:?}", push_step);
        }

        Ok(())
    }

    pub async fn push(&self, ctx: &DeployableContext) -> Result<()> {
        let Some(container) = &self.container else {
            return Ok(());
        };

        for ct in container.keys() {
            self.push_ct(ctx, &ct).await?;
        }

        Ok(())
    }

    pub async fn pull_ct(&self, ctx: &DeployableContext, ct: &str) -> Result<()> {
        let options = CreateImageOptionsBuilder::new()
            .from_image(&self.image_id(ctx, ct))
            .build();
        let mut pull = ctx
            .docker
            .create_image(Some(options), None, ctx.docker_credentials.clone());

        while let Some(pull_step) = pull.next().await {
            let pull_step = pull_step.context("Docker image pull error")?;
            info!("{:?}", pull_step);
        }

        Ok(())
    }

    pub async fn pull(&self, ctx: &DeployableContext) -> Result<()> {
        let Some(container) = &self.container else {
            return Ok(());
        };

        for ct in container.keys() {
            self.pull_ct(ctx, &ct).await?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Flag {
    Raw(String),
    File { file: PathBuf },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Attachment {
    File(PathBuf),
    Named {
        file: PathBuf,
        r#as: String,
    },
    Archive {
        dir: PathBuf,
        // without archive extension
        #[serde(default = "default_archive_name")]
        r#as: String,
        #[serde(default)]
        exclude: Option<Vec<PathBuf>>,
    },
}

fn default_archive_name() -> String {
    "chall".to_owned()
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Container {
    pub build: PathBuf,
    #[serde(default)]
    pub limits: Limits,
    pub env: Option<HashMap<String, String>>,
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    pub expose: Option<HashMap<u16, ExposeType>>,
    pub cap_add: Option<Vec<String>>,
    pub privileged: Option<bool>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentStrategy {
    Static,
    Instanced,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Limits {
    // "nano-cpus", i.e. in units of 10^-9 cpu
    pub cpu: Option<i64>,
    // in bytes
    pub mem: Option<i64>,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            cpu: Some(1_000_000_000), // 1vcpu
            mem: Some(104_857_600),   // 100mb
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExposeType {
    Tcp,
    Http,
}

#[derive(Debug, Clone)]
pub struct DeployableChallenge {
    pub chall: Challenge,
    pub root: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Serializable version of [super::DeployableContext].
pub struct DeployableContextData {
    // TODO(aiden): rename to daemon
    pub docker: DockerData,
    pub docker_credentials: Option<bollard::auth::DockerCredentials>,
    // TODO(aiden): image_prefix and repo are basically the same thing iirc? get rid of image_prefix
    pub image_prefix: String,
    pub repo: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum DockerData {
    /// use local defaults
    Local,
    /// use ssl
    Ssl {
        address: String,
        // in pem
        key: String,
        cert: String,
        ca: String,
    },
}

impl TryInto<bollard::Docker> for DockerData {
    type Error = bollard::errors::Error;

    fn try_into(self) -> std::result::Result<bollard::Docker, Self::Error> {
        match self {
            Self::Local => bollard::Docker::connect_with_local_defaults(),
            Self::Ssl {
                address,
                key,
                cert,
                ca,
            } => {
                // good enough?
                let basedir = std::env::temp_dir().join(format!(
                    "docker-certs-dir-{}",
                    address.replace(":", "-").replace(".", "-")
                ));
                std::fs::create_dir_all(&basedir).unwrap();
                // FIXME(ani): avoid unwraps
                let key_path = basedir.join("key.pem");
                let cert_path = basedir.join("cert.pem");
                let ca_path = basedir.join("ca.pem");
                fs::write(&key_path, key).unwrap();
                fs::write(&cert_path, cert).unwrap();
                fs::write(&ca_path, ca).unwrap();
                bollard::Docker::connect_with_ssl(
                    &address,
                    &key_path,
                    &cert_path,
                    &ca_path,
                    120,
                    bollard::API_DEFAULT_VERSION,
                )
            }
        }
    }
}

impl TryInto<DeployableContext> for DeployableContextData {
    type Error = bollard::errors::Error;

    fn try_into(self) -> std::result::Result<DeployableContext, Self::Error> {
        Ok(DeployableContext {
            docker: self.docker.try_into()?,
            docker_credentials: self.docker_credentials,
            image_prefix: self.image_prefix,
            repo: self.repo,
        })
    }
}

#[derive(Debug, Clone)]
/// Expected to be passed by callers.
pub struct DeployableContext {
    pub docker: bollard::Docker,
    pub docker_credentials: Option<bollard::auth::DockerCredentials>,
    pub image_prefix: String,
    pub repo: String,
}

pub fn is_valid_id(id: &str) -> bool {
    id.chars()
        .all(|c| (!c.is_uppercase() && c.is_alphanumeric()) || c == '-')
}

impl DeployableChallenge {
    pub fn from_root(root: PathBuf) -> Result<Self> {
        let chall_data = fs::read_to_string(root.join("challenge.toml"))
            .with_context(|| format!("Failed to read challenge.toml in {}", root.display()))?;
        let chall = toml::from_str::<Challenge>(&chall_data)?;

        if !is_valid_id(&chall.id) {
            return Err(eyre!("Id must be lowercase alphanumeric with -"));
        }

        Ok(Self { chall, root })
    }

    pub async fn build_ct(
        &self,
        ctx: &DeployableContext,
        ct: &str,
    ) -> Result<Option<Vec<bollard::models::BuildInfo>>> {
        let Some(chall_containers) = &self.chall.container else {
            return Ok(None);
        };

        let Some(chall_container) = chall_containers.get(ct) else {
            return Ok(None);
        };

        let tmp = TempDir::new(&self.chall.id)?;
        let context_tar_path = tmp.path().join("docker.tar");
        {
            // ugh
            let tar_file = StdFile::create(&context_tar_path)?;
            let mut tar_ = tar::Builder::new(tar_file);
            tar_.sparse(false);
            let context_dir_path = self.root.join(&chall_container.build);
            tar_.append_dir_all(".", &context_dir_path)
                .with_context(|| {
                    format!(
                        "Failed to read Docker context {}",
                        context_dir_path.display()
                    )
                })?;
            tar_.finish()?;
        }

        let options = bollard::query_parameters::BuildImageOptionsBuilder::new()
            // FIXME(ani): idk if it's ideal to tag the image with the repo name in build
            .t(&self.chall.image_id(ctx, ct))
            .forcerm(true)
            .rm(true)
            .build();
        let tar_file_r = File::open(&context_tar_path).await?;
        let tar_file_r = ReaderStream::new(tar_file_r);
        let mut build =
            ctx.docker
                .build_image(options, None, Some(bollard::body_try_stream(tar_file_r)));
        let mut build_infos = vec![];
        while let Some(build_step) = build.next().await {
            let build_step = build_step.context("Docker build image error")?;
            if let Some(stream) = &build_step.stream {
                info!("{}", stream);
            }
            build_infos.push(build_step);
        }

        Ok(Some(build_infos))
    }

    pub async fn build(
        &self,
        ctx: &DeployableContext,
    ) -> Result<Vec<Vec<bollard::models::BuildInfo>>> {
        let Some(container) = &self.chall.container else {
            return Ok(vec![]);
        };

        let mut out = vec![];
        for ct in container.keys() {
            if let Some(b) = self.build_ct(ctx, &ct).await? {
                out.push(b);
            }
        }

        Ok(out)
    }

    // compat
    pub async fn pull(&self, ctx: &DeployableContext) -> Result<()> {
        self.chall.pull(ctx).await
    }
    pub async fn push(&self, ctx: &DeployableContext) -> Result<()> {
        self.chall.push(ctx).await
    }

    pub async fn push_attachments(
        &self,
        client: &Client,
        bucket: String,
    ) -> Result<HashMap<String, String>> {
        if self.chall.provide.is_none() {
            return Ok(HashMap::new());
        }

        let mut hm = HashMap::new();
        for attachment in self.chall.provide.as_ref().unwrap() {
            let (name, data) = match attachment {
                Attachment::File(path) => {
                    let name = path.file_name().unwrap().to_str().unwrap().to_owned();
                    let data = fs::read_to_string(self.root.join(path))?;
                    (name, Vec::from(data.as_bytes()))
                }
                Attachment::Named { file, r#as } => {
                    let data = fs::read_to_string(self.root.join(file))?;
                    (r#as.clone(), Vec::from(data.as_bytes()))
                }
                Attachment::Archive { dir, r#as, exclude } => {
                    let tmp = TempDir::new(&self.chall.id)?;
                    let tar_path = tmp.path().join("chall.tar.gz");

                    // ugh
                    {
                        let tar_file = StdFile::create(&tar_path)?;
                        let enc = GzEncoder::new(tar_file, Compression::default());
                        let mut tar_ = tar::Builder::new(enc);
                        // TODO actually support exclude
                        tar_.append_dir_all(r#as, self.root.join(dir))?;

                        tar_.finish()?;
                    }

                    let mut buffer = Vec::new();
                    StdFile::open(&tar_path)?.read_to_end(&mut buffer)?;

                    (format!("{as}.tar.gz"), buffer)
                }
            };

            let upload_type =
                UploadType::Simple(Media::new(format!("{}/{}", self.chall.id, &name)));

            let uploaded = client
                .upload_object(
                    &UploadObjectRequest {
                        bucket: bucket.clone(),
                        ..Default::default()
                    },
                    data,
                    &upload_type,
                )
                .await?;

            let url_for_download = client
                .signed_url(
                    &bucket,
                    &uploaded.name,
                    None,
                    None,
                    SignedURLOptions {
                        expires: Duration::from_secs(604800),
                        ..Default::default()
                    },
                )
                .await?;

            hm.insert(name, url_for_download);
        }
        return Ok(hm);
    }
}
