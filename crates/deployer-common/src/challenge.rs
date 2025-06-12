use bollard::{image::CreateImageOptions, query_parameters::CreateImageOptionsBuilder};
use eyre::{Context, Result, eyre};
use log::info;
use serde_with::{DisplayFromStr, serde_as};
use std::{
    collections::HashMap,
    fs::{self, File as StdFile},
    path::PathBuf,
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
    pub group: Option<String>,
    pub category: String,
    pub provide: Option<Vec<Attachment>>,
    pub container: Option<Container>,
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
        globs: Vec<String>,
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
    pub limits: Option<Limits>,
    pub env: Option<HashMap<String, String>>,
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    pub expose: Option<HashMap<u16, ExposeType>>,
    #[serde(default = "default_strategy")]
    pub strategy: ContainerStrategy,
}

fn default_strategy() -> ContainerStrategy {
    ContainerStrategy::Static
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerStrategy {
    Static,
    Instanced,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Limits {
    pub cpu: Option<u64>,
    pub mem: Option<u64>,
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

#[derive(Debug, Clone)]
/// Expected to be passed by callers.
pub struct DeployableContext {
    pub docker: bollard::Docker,
    pub docker_credentials: Option<bollard::auth::DockerCredentials>,
    pub image_prefix: String,
}

pub fn is_valid_id(id: &str) -> bool {
    id.chars().all(|c| (!c.is_uppercase() && c.is_alphanumeric()) || c == '-')
}

impl DeployableChallenge {
    pub fn from_root(root: PathBuf) -> Result<Self> {
        let chall_data = fs::read_to_string(root.join("challenge.toml"))
            .with_context(|| format!("Failed to read challenge.toml in {}", root.display()))?;
        let chall = toml::from_str::<Challenge>(&chall_data)?;

        if !is_valid_id(&chall.id) {
            return Err(eyre!("Id must be alphanumeric with -"));
        }

        Ok(Self { chall, root })
    }

    fn image_id(&self, ctx: &DeployableContext) -> String {
        format!("{}{}", ctx.image_prefix, self.chall.id)
    }

    pub async fn build(
        &self,
        ctx: &DeployableContext,
    ) -> Result<Option<Vec<bollard::models::BuildInfo>>> {
        let Some(chall_container) = &self.chall.container else {
            return Ok(None);
        };

        let tmp = TempDir::new(&self.chall.id)?;
        let context_tar_path = tmp.path().join("docker.tar");
        {
            // ugh
            let tar_file = StdFile::create(&context_tar_path)?;
            let mut tar_ = tar::Builder::new(tar_file);
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
            .t(&self.image_id(ctx))
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

    pub async fn push(&self, ctx: &DeployableContext) -> Result<()> {
        // TODO: support credentials
        let mut push = ctx.docker.push_image(
            &self.image_id(ctx),
            None::<bollard::query_parameters::PushImageOptions>,
            ctx.docker_credentials.clone(),
        );

        while let Some(push_step) = push.next().await {
            let push_step = push_step.context("Docker image push error")?;
            info!("{:?}", push_step);
        }

        Ok(())
    }

    pub async fn pull(&self, ctx: &DeployableContext) -> Result<()> {
        let options = CreateImageOptionsBuilder::new()
            .repo("gcr.io")
            .from_src(&self.image_id(ctx))
            .build();
        let mut pull = ctx
            .docker
            .create_image(Some(options), None, ctx.docker_credentials.clone());

        while let Some(pull_step) = pull.next().await {
            let pull_step = pull_step.context("Docker image push error")?;
            info!("{:?}", pull_step);
        }

        Ok(())
    }

    pub async fn push_attachments(&self) -> Result<()> {
        /* build and push attachments to s3/gcs */
        todo!()
    }
}
