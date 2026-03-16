use clap::Parser;

#[derive(Parser, Default, Clone, Debug)]
#[clap(name = "crousd", version, about = "Hack The Crous server")]
pub struct Config {
    #[clap(env, long, default_value = "3000", help = "Port to listen on")]
    pub port: u16,

    #[clap(env, long, default_value = "hackthecrous.com", help = "Allowed origins")]
    pub origins: Vec<String>,

    #[clap(env, long, help = "Database URL")]
    pub database_url: String,

    #[clap(env, long, help = "Default admin public key")]
    pub admin_public_key: String
}

