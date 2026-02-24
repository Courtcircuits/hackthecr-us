use clap::Parser;

#[derive(Parser, Default, Clone, Debug)]
#[clap(name = "crousd", version, about = "Hack The Crous server")]
pub struct Config {
    #[clap(env, long, default_value = "3000", help = "Port to listen on")]
    pub port: u16,

    #[clap(env, long, default_value = "beep.com", help = "Allowed origins")]
    pub origins: Vec<String>,

    #[clap(env, long, help = "Database URL")]
    pub database_url: String
}

