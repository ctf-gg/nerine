use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand, command};
use deployer_common::challenge::{Challenge, Container, ExposeType, Flag, is_valid_id};
use dialoguer::{Select, theme::SimpleTheme};
use eyre::Result;
use rustyline::DefaultEditor;
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
}
// todo case sensitive or not?
fn search_for(dir: &Path, filenames: &[&str]) -> Option<PathBuf> {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let current_filename = &entry.file_name().to_str()?;
        if filenames.iter().any(|f| f == current_filename) {
            return Some(entry.path().to_owned());
        }
    }
    None
}

fn main() -> Result<()> {
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
                        .to_string()
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
                    docker_path
                } else {
                    println!("No Dockerfile found, defaulting to ./Dockerfile");
                    PathBuf::from("./Dockerfile")
                };

            let expose_type_selection = Select::with_theme(&SimpleTheme)
                .with_prompt("How is your challenge exposed?")
                .default(0)
                .items(&["TCP", "HTTP"])
                .interact()?;
            let expose_type: ExposeType =
                [ExposeType::Tcp, ExposeType::Http][expose_type_selection].clone();

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

            // let mut expose = HashMap::new();
            // expose.insert(expose_port, expose_type);

            let container = Some(Container {
                build: dockerfile_path.strip_prefix(&path).unwrap_or(&dockerfile_path).to_owned(),
                limits: None,
                env: None,
                expose: Some({
                    let mut m = HashMap::new();
                    m.insert(expose_port, expose_type);
                    m
                }),
            });

            let chall = Challenge {
                id,
                name,
                flag,
                author: "You!".to_string(),
                group: None,
                category: path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|f| f.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                description: "challenge description".to_string(),
                container,
                provide: None,
            };

            path.push("challenge.toml");
            let mut file = File::create(&path)?;
            write!(file, "{}", toml::to_string_pretty(&chall)?)?;

            println!("Created {}", path.to_str().unwrap_or("challenge.toml"));
        }
    }
    Ok(())
}
