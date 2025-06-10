use std::{cell::LazyCell, env, path::PathBuf};

use bollard::Docker;
use challenge::Challenge;
use eyre::OptionExt;

mod challenge;

pub const DOCKER: LazyCell<Docker> =
    LazyCell::new(|| Docker::connect_with_local_defaults().expect("failed to connect to docker"));

fn main() -> eyre::Result<()> {
    let mut args = env::args();
    args.next();

    let root = PathBuf::from(args.next().ok_or_eyre("must pass argument to builder")?);

    let challs = Challenge::get_all(&root);

    println!("{challs:?}");

    Ok(())
}
