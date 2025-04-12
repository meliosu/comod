use anyhow::{anyhow, bail};

#[derive(clap::Parser)]
pub struct Args {
    #[arg(long)]
    pub model: String,

    #[arg(long)]
    pub ucodes: String,

    #[arg(long)]
    pub debug: bool,

    #[arg(long, value_parser = parse_pair, value_delimiter = ',')]
    pub inputs: Vec<(String, String)>,

    #[arg(long, value_delimiter = ',')]
    pub outputs: Vec<String>,
}

fn parse_pair(input: &str) -> anyhow::Result<(String, String)> {
    input
        .split_once("=")
        .map(|(a, b)| (a.into(), b.into()))
        .ok_or(anyhow!("error parsing kv pairs"))
}

fn parse_inputs(inputs: &str) -> anyhow::Result<Vec<(String, String)>> {
    let mut result = Vec::new();

    for pair in inputs.split(",") {
        let Some((key, value)) = pair.split_once("=") else {
            bail!("error parsing key-value pairs");
        };

        result.push((key.into(), value.into()));
    }

    Ok(result)
}
