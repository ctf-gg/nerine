use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use bollard::auth::DockerCredentials;
use clap::{Parser, Subcommand, command};
use deployer_common::challenge::{
    Challenge, Container, DeployableChallenge, DeployableContext, DeploymentStrategy, ExposeType,
    Flag, is_valid_id,
};
use dialoguer::{Select, theme::SimpleTheme};
use eyre::{Result, eyre};
use google_cloud_storage::client::{Client as GcsClient, ClientConfig};
use reqwest::{Client, Url, cookie::Jar};
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use walkdir::WalkDir;

#[derive(Debug, Parser)]
#[command(name = "sctf")]
#[command(about = "Tool for managing challenges with sctf", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    Build {
        #[arg()]
        paths: Vec<PathBuf>,

        /// Specifies which build group to use
        #[arg(short = 'g', long)]
        build_group: Option<String>,

        /// Builds all challenges regardless of build group
        #[arg(short, long)]
        all: bool,

        /// Exits without building if there are any toml parse errors
        #[arg(short, long)]
        strict: bool,
    },

    Platform {
        #[command(subcommand)]
        command: PlatformCommands,
    },
    CoalesceManifests {
        #[arg()]
        dir: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum PlatformCommands {
    Update {
        #[arg()]
        paths: Vec<PathBuf>,
        /// Specifies which build group to use
        #[arg(short = 'g', long)]
        build_group: Option<String>,
        /// Skip pushing attachments to gcs and make all attachments empty in db
        #[arg(short = 'n', long)]
        null_attachments: bool,
    },
}
// todo case sensitive or not?
fn search_for(dir: &Path, filenames: &[&str]) -> Option<PathBuf> {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let current_filename = &entry.file_name();
        if filenames.iter().any(|f| f == current_filename) {
            return Some(entry.path().to_owned());
        }
    }
    None
}

fn get_all_challs(paths: Vec<PathBuf>) -> impl Iterator<Item = DeployableChallenge> {
    let chall_paths: Vec<PathBuf> = if paths.len() == 0 {
        WalkDir::new(".")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "challenge.toml")
            .map(|e| e.path().parent().unwrap().to_owned())
            .collect()
    } else {
        paths.clone()
    };

    let challs: Vec<Result<DeployableChallenge>> = chall_paths
        .into_iter()
        .map(|p| {
            DeployableChallenge::from_root(p.clone()).map_err(|err| {
                eyre!(
                    "at {}:\n{}",
                    p.join("challenge.toml").to_str().unwrap().to_string(),
                    err.to_string()
                )
            })
        })
        .collect();

    let parse_errors: Vec<&eyre::ErrReport> =
        challs.iter().filter_map(|c| c.as_ref().err()).collect();
    if parse_errors.len() > 0 {
        eprintln!("Toml errors:");
        for err in parse_errors {
            eprintln!("{}", err)
        }
    }

    return challs.into_iter().filter_map(|c| c.ok());
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    _ = dotenvy::dotenv(); // we don't care whether its there or not
    let args = Cli::parse();
    match args.command {
        Commands::Init { mut path } => {
            // TODO currently it doesn't know what the challenges root is.
            let mut rl = DefaultEditor::new()?;

            let inferred_id = path
                .to_str()
                .map(|p| {
                    p.replace("./", "")
                        .replace("/", "-")
                        .trim_matches('-')
                        .to_ascii_lowercase()
                })
                .filter(|p| is_valid_id(p));

            if let Some(id) = inferred_id.as_ref() {
                println!("Inferred id to be {id}");
            }

            // should be safe due to canonicalization
            let name = path
                .canonicalize()?
                .file_name()
                .unwrap()
                .to_str()
                .expect("Path is invalid utf-8")
                .to_string();

            let id = if let Some(i) = inferred_id {
                i
            } else {
                rl.readline("Enter a unique id for your challenge: ")?
            };

            let flag: Flag = if let Some(flag_path) = search_for(&path, &["flag.txt", "flag"]) {
                println!("Found {}, using as flag file", flag_path.to_string_lossy());
                Flag::File {
                    file: flag_path.strip_prefix(&path)?.to_owned(),
                }
            } else {
                println!("No flag found, using example flag");
                Flag::Raw("example_flag".to_string())
            };

            let dockerfile_path: PathBuf =
                if let Some(docker_path) = search_for(&path, &["Dockerfile"]) {
                    println!(
                        "Found {}, using as container Dockerfile",
                        docker_path.to_string_lossy()
                    );
                    docker_path.parent().unwrap().to_owned()
                } else {
                    println!("No Dockerfile found, defaulting to ./");
                    PathBuf::from(".")
                };

            let expose_type_selection = Select::with_theme(&SimpleTheme)
                .with_prompt("How is your challenge exposed?")
                .default(0)
                .items(&["TCP", "HTTP"])
                .interact()?;
            let expose_type = [ExposeType::Tcp, ExposeType::Http][expose_type_selection];

            let expose_port: u16 = {
                loop {
                    let line = rl.readline("What port does your container expose? ")?;
                    if let Ok(port) = line.parse::<u16>() {
                        break port;
                    } else {
                        eprintln!("Enter a valid port.")
                    }
                }
            };

            let container_strategy_selection = Select::with_theme(&SimpleTheme)
                .with_prompt(
                    "Does your container have one instance for everyone, or one instance per team?",
                )
                .default(0)
                .items(&["Static (one for everyone)", "Instanced (one per team)"])
                .interact()?;
            let container_strategy = [DeploymentStrategy::Static, DeploymentStrategy::Instanced]
                [container_strategy_selection];

            // let mut expose = HashMap::new();
            // expose.insert(expose_port, expose_type);

            let container = Container {
                build: dockerfile_path
                    .strip_prefix(&path)
                    .unwrap_or(&dockerfile_path)
                    .to_owned(),
                limits: None,
                env: None,
                expose: Some({
                    let mut m = HashMap::new();
                    m.insert(expose_port, expose_type);
                    m
                }),
                privileged: None,
            };

            let chall = Challenge {
                id,
                name,
                flag,
                author: "You!".to_string(),
                visible: None,
                group: None,
                build_group: None,
                category: path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|f| f.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                description: "challenge description".to_string(),
                container: Some({
                    // FIXME
                    let mut m = HashMap::new();
                    m.insert("default".to_owned(), container);
                    m
                }),
                strategy: container_strategy,
                provide: None,
                host: None,
            };

            path.push("challenge.toml");
            let mut file = File::create(&path)?;
            write!(file, "{}", toml::to_string_pretty(&chall)?)?;

            println!("Created {}", path.to_str().unwrap_or("challenge.toml"));
        }
        Commands::Build {
            paths,
            build_group,
            all,
            strict,
        } => {
            let valid_challs: Vec<DeployableChallenge> = get_all_challs(paths)
                .filter(|c| c.chall.container.is_some())
                .filter(|c| all || c.chall.build_group == build_group)
                .collect();
            println!("Building following challenges:");
            for chall in &valid_challs {
                println!("{}", chall.chall.id)
            }

            let ctx = DeployableContext {
                docker: bollard::Docker::connect_with_local_defaults()?,
                // TODO if something not found, default to None
                docker_credentials: Some(DockerCredentials {
                    username: Some(env::var("DOCKER_USERNAME")?),
                    password: Some(env::var("DOCKER_PASSWORD")?),
                    email: None,
                    serveraddress: Some(env::var("DOCKER_SERVERADDRESS")?),
                    ..Default::default()
                }),
                // docker_credentials: None,
                image_prefix: "".to_string(),
                repo: env::var("DOCKER_REPO")?,
            };

            for chall in valid_challs {
                println!("building chall {}", chall.chall.id);
                // TODO i suspect this might not be working
                chall.pull(&ctx).await?;
                match chall.build(&ctx).await {
                    Ok(_) => {
                        println!("pushing chall {}", chall.chall.id);
                        chall.push(&ctx).await?;
                    }
                    Err(e) => eprintln!("failed to build {}: {e:?}", chall.chall.id),
                };
            }
        }
        Commands::Platform { command } => match command {
            PlatformCommands::Update { paths, build_group, null_attachments } => {
                #[derive(Deserialize, Serialize)]
                pub struct Category {
                    pub id: i32,
                    pub name: String,
                }

                #[derive(Serialize)]
                pub struct UpsertChallenge {
                    pub id: Option<String>,
                    pub name: String,
                    pub author: String,
                    pub description: String,
                    pub points_min: i32,
                    pub points_max: i32,
                    pub flag: String,
                    pub attachments: serde_json::Value,
                    pub strategy: DeploymentStrategy,
                    pub visible: bool,

                    pub category_id: i32,
                    pub group_id: Option<i32>,
                }

                let platform_base = env::var("PLATFORM_BASE")?;
                let jar = Jar::default();
                jar.add_cookie_str(
                    &format!("admin_token={}", env::var("PLATFORM_ADMIN_TOKEN")?),
                    &Url::parse(&platform_base)?,
                );
                let client = reqwest::Client::builder()
                    .cookie_provider(Arc::new(jar))
                    .build()?;
                let mut categories: HashMap<String, i32> = client
                    .get(format!("{platform_base}/api/admin/challs/category"))
                    .send()
                    .await?
                    .error_for_status()?
                    .json::<Vec<Category>>()
                    .await?
                    .into_iter()
                    .map(|c| (c.name, c.id))
                    .collect();

                let gcs_client = if null_attachments {
                    None
                } else {
                    Some(GcsClient::new(ClientConfig::default().with_auth().await.unwrap()))
                };
                for ref dc in
                    get_all_challs(paths).filter(|c| c.chall.build_group == build_group)
                {
                    let DeployableChallenge { chall, root } = dc;
                    let attachments = if null_attachments {
                        HashMap::new()
                    } else {
                        dc.push_attachments(gcs_client.as_ref().unwrap(), "sctf-attachments".to_string()).await?
                    };
                    client
                        .patch(format!("{platform_base}/api/admin/challs"))
                        .json(&UpsertChallenge {
                            id: Some(chall.id.clone()),
                            name: chall.name.clone(),
                            author: chall.author.clone(),
                            description: chall.description.clone(),
                            points_max: 500,
                            points_min: 100,
                            flag: match chall.flag.clone() {
                                Flag::Raw(flag) => flag,
                                Flag::File { file } => {
                                    fs::read_to_string(root.join(file))?.trim().to_string()
                                }
                            },
                            attachments: attachments.serialize(serde_json::value::Serializer)?,
                            strategy: chall.strategy,
                            visible: chall.visible != Some(false),
                            category_id: match categories.get(&chall.category) {
                                Some(c) => *c,
                                None => {
                                    #[derive(Serialize)]
                                    struct CreateCategory {
                                        name: String,
                                    }

                                    let new_category: Category = client
                                        .post(format!("{platform_base}/api/admin/challs/category"))
                                        .json(&CreateCategory {
                                            name: chall.category.clone(),
                                        })
                                        .send()
                                        .await?
                                        .json()
                                        .await?;

                                    categories.insert(new_category.name, new_category.id);

                                    new_category.id
                                }
                            },
                            group_id: None,
                        })
                        .send()
                        .await?
                        .error_for_status()?;

                    println!("updated {}", chall.id);
                }

                client.post(format!("{platform_base}/api/admin/challs/reload_deployer")).send().await?.error_for_status()?;
                println!("reloaded deployer")
            }
        },
        Commands::CoalesceManifests { dir } => {
            _ = fs::create_dir(&dir);

            for mf in WalkDir::new(".")
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name() == "challenge.toml")
            {
                fs::copy(
                    mf.path(),
                    &dir.join(
                        mf.path()
                            .to_str()
                            .unwrap()
                            .replace("/", "-")
                            .trim_matches('.')
                            .trim_matches('-'),
                    ),
                )?;
            }
        }
    }
    Ok(())
}
