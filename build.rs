use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use std::{fs, path, process::exit};

include!("src/cli.rs");

fn main() -> Result<(), clap::Error> {
    let outdir = path::Path::new(option_env!("OUT_DIR").unwrap_or_else(|| {
        exit(0);
    }))
    .join("completions/");

    if !outdir.exists() {
        fs::create_dir(outdir.clone()).expect("Failed to create 'completions' directory.");
    }

    let mut cmd = Opt::command();

    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, "gradient", outdir.clone())?;
    }

    Ok(())
}
