use clap::Parser;

#[derive(Parser, Default, Clone, Debug)]
#[clap(name = "gateway", version, about = "Gateway for the buffet")]
pub struct Config {
    /// Port to listen on
    #[clap(long, env = "GATEWAY_PORT", default_value = "8080")]
    pub port: u16,

    /// Allowed CORS origins (comma-separated, use '*' for all)
    #[clap(long, env = "GATEWAY_CORS_ORIGINS", default_value = "*")]
    pub cors_origins: Vec<String>,

    #[clap(long, env = "CONSUMER_ENDPOINT")]
    pub consumer_endpoint: String,

    #[clap(long, env = "REDIS_URL")]
    pub redis_url: String,
}
