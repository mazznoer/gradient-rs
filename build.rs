use clap::*;
use clap_complete::{generate_to, Shell};
use std::env;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("completions/");
    let mut cmd = Opt::command();

    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, "gradient", outdir.clone())?;
    }

    Ok(())
}
