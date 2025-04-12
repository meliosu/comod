#[derive(clap::Parser)]
pub struct Args {
    #[arg(long)]
    pub model: String,

    #[arg(long)]
    pub ucodes: String,

    #[arg(long)]
    pub debug: bool,

    #[arg(long, value_delimiter = ',')]
    pub inputs: Vec<String>,

    #[arg(long, value_delimiter = ',')]
    pub outputs: Vec<String>,
}
