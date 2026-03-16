use palc::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    version,
    long_about = "NodeGet is the next-generation server monitoring and management tools. nodeget-agent is a part of it",
    after_long_help = "This Server is open-sourced on Github, powered by powerful Rust. Love from NodeGet"
)]
pub struct ServerArgs {
    #[arg(long, short, default_value_t = "config.toml".to_string())]
    pub config: String,
}

impl ServerArgs {
    pub fn par() -> Self {
        let args = Self::parse();
        // todo: add check
        args
    }
}