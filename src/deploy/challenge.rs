use eyre::{eyre, Result};
use serde::Deserialize;
use std::{collections::HashMap, fs, io, path::PathBuf};

#[derive(Deserialize, Debug, Clone)]
pub struct Challenge {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub flag: Flag,
    pub hidden: Option<bool>,
    pub provide: Option<Vec<Attachment>>,
    #[serde(default = "empty")]
    pub containers: HashMap<String, Container>,
    #[serde(default = "empty")]
    pub expose: HashMap<String, Expose>,
}

fn empty<K, V>() -> HashMap<K, V> {
    HashMap::new()
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Flag {
    Raw(String),
    File { file: PathBuf },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Attachment {
    File(PathBuf),
    Named {
        file: PathBuf,
        r#as: String,
    },
    Folder {
        dir: PathBuf,
        r#as: Option<String>,
        exclude: Option<Vec<PathBuf>>,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct Container {
    pub build: PathBuf,
    pub limits: Limits,
    ports: Option<Vec<u32>>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Limits {
    pub cpu: Option<u32>,
    pub mem: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Expose {
    Tcp { target: u32, tcp: u32 },
    Http { target: u32, http: String },
}

impl Challenge {
    // TODO(aiden): this needs to be made agnostic and just take in a toml string
    pub fn parse(chall_dir: PathBuf) -> Result<Challenge> {
        let file_data = fs::read_to_string(&chall_dir.join("challenge.toml")).map_err(|_| eyre!("Failed to read challenge.toml, Make sure it exists at the root of your challenge directory"))?;
        let mut toml: toml::Table = toml::from_str(&file_data)?;
        let mut id_parts = chall_dir
            .iter()
            .rev()
            .take(2)
            .map(|c| c.to_str())
            .collect::<Option<Vec<&str>>>()
            .ok_or(eyre!("Failed to convert OsStr to Str"))?;
        id_parts.reverse();
        let id = id_parts.join("/");
        toml.insert(String::from("id"), toml::Value::String(id.clone()));

        Ok(toml
            .try_into()
            .map_err(|e| eyre!("failed to parse parsing {id}: {e}"))?)
    }

    pub fn get(root: &PathBuf, chall: String) -> Result<Challenge> {
        let dir = root.as_path().join(chall);
        if !dir.is_dir() {
            return Err(eyre!("{} is not a directory", dir.display()));
        }

        Challenge::parse(dir)
    }

    pub fn get_all(root: &PathBuf) -> Result<Vec<Challenge>> {
        let paths = get_chall_paths(root)?;
        paths
            .into_iter()
            .map(|path| Challenge::parse(path))
            .collect::<Result<Vec<Challenge>, _>>()
    }

    pub fn get_some(root: &PathBuf, challs: Vec<String>) -> Result<Vec<Challenge>> {
        let mut parsed_challs = Vec::new();
        for chall in challs {
            parsed_challs.push(Challenge::get(&root, chall)?);
        }
        Ok(parsed_challs)
    }

    // TODO return type bad >:(
    // pub async fn build(
    //     self,
    //     root: &PathBuf,
    //     tmp_dir: &TempDir,
    // ) -> Result<Vec<bollard::models::BuildInfo>> {
    //     let build_info = vec![];
    //     for (name, container) in &self.containers {
    //         let mut build_path = root.clone();
    //         build_path.push(&self.id);
    //         build_path.push(&container.build);

    //         // build a tar
    //         let tar_path =
    //             tmp_dir.child(format!("{}-{}.docker.tar", self.id.replace("/", "-"), name));
    //         let tar_file = File::create(&tar_path)?;
    //         let mut tar = tar::Builder::new(tar_file);
    //         tar.append_dir_all(".", &build_path).map_err(|e| {
    //             anyhow!(
    //                 "Failed to read {}. Make sure it exists and is a directory.\n{e:?}",
    //                 build_path.display()
    //             )
    //         })?;
    //         tar.finish()?;

    //         let options = BuildImageOptions {
    //             dockerfile: "Dockerfile",
    //             t: &self.container_id(name),
    //             rm: true,
    //             ..Default::default()
    //         };
    //         let mut read_tar = File::open(&tar_path)?;
    //         let mut contents = Vec::new();
    //         read_tar.read_to_end(&mut contents).unwrap();
    //         println!("building image: {}", options.t);
    //         let mut build = DOCKER.build_image(options, None, Some(contents.into()));
    //         while let Some(build_step) = build.next().await {
    //             if let Some(stream) = build_step
    //                 .map_err(|err| anyhow!("docker error: {err:?}"))?
    //                 .stream
    //             {
    //                 print!("{stream}")
    //             }
    //         }

    //         print!("\n");

    //         // build_info.push();
    //     }
    //     Ok(build_info)
    // }

    // pub async fn push(&self, repo: &str, name: &str) -> Result<()> {
    //     let image_name = self.container_id(name);
    //     let new_tag = format!("{repo}:{image_name}");
    //     DOCKER
    //         .tag_image(
    //             &image_name,
    //             Some(bollard::image::TagImageOptions {
    //                 repo: new_tag.clone(),
    //                 ..Default::default()
    //             }),
    //         )
    //         .await?;
    //     let mut push = DOCKER.push_image::<String>(
    //         &new_tag,
    //         None,
    //         Some(DockerCredentials {
    //             // https://community.fly.io/t/push-to-fly-io-image-registry-via-docker-api/9132
    //             // I have NO idea why the username is x and why the password is the api token,
    //             // this took 2 hours to figure out and probably took a couple years off my life as well.
    //             username: Some("x".to_string()),
    //             password: Some(fly::FLY_API_TOKEN.clone()),
    //             ..Default::default()
    //         }),
    //     );

    //     while let Some(push_step) = push.next().await {
    //         println!("{:#?}", push_step);
    //     }
    //     Ok(())
    // }

    // pub async fn push_all(&self, repo: &str) -> Result<()> {
    //     for (name, _) in &self.containers {
    //         self.push(repo, name).await?;
    //     }

    //     Ok(())
    // }

    pub fn container_id(&self, name: &str) -> String {
        return format!("{}-{}", self.id.replace("/", "-"), name);
    }

    pub fn get_flag(&self, root: &PathBuf) -> Result<String> {
        match &self.flag {
            Flag::Raw(flag) => Ok(flag.clone()),
            Flag::File { file } => Ok(fs::read_to_string(root.join(&self.id).join(file))?),
        }
    }
}

pub fn get_chall_paths(root: &PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut challenges = Vec::new();
    for category_entry in fs::read_dir(root)? {
        let category = category_entry?.path();

        if category.is_dir() {
            for chall_entry in fs::read_dir(&category)? {
                let chall = chall_entry?.path();
                let path = &chall.join("challenge.toml");
                if path.exists() {
                    challenges.push(chall);
                }
            }
        }
    }
    return Ok(challenges);
}
