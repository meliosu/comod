use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, bail};
use clap::Parser;

use comod::{args::Args, codegen::Generator, compile, format, types::Model};

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
    }
}

fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    let model = std::fs::read_to_string(&args.model).map_err(|e| anyhow!("reading model: {e}"))?;

    let model: Model = serde_json::from_str(&model).map_err(|e| anyhow!("parsing model: {e}"))?;

    let ucodes =
        std::fs::read_to_string(&args.ucodes).map_err(|e| anyhow!("reading ucodes: {e}"))?;

    let generator = Generator::new(model, ucodes);

    let code = generator
        .generate()
        .map_err(|e| anyhow!("failed to generate code: {e}"))?;

    if args.debug {
        let code = format::format(&code).map_err(|e| anyhow!("formatting code: {e}"))?;
        println!("{code}");
    }

    let name = PathBuf::from_str(&args.model)?;

    let Some(name) = name.file_name().and_then(|s| s.to_str()) else {
        bail!("model needs to have .json extension");
    };

    compile::compile(&code, name).map_err(|e| anyhow!("compiling code: {e}"))?;

    Ok(())
}
