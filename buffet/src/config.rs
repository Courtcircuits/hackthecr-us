use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Default, Clone, Debug)]
#[clap(name = "buffet", version, about = "Buffet, a scheduler for gourmets web scrapers")]
pub struct Config {
    #[clap(long, env = "PRODUCER_ENDPOINT")]
    pub producer_endpoint: String,

    #[clap(long, env = "REDIS_URL")]
    pub redis_url: String,

    #[clap(long, env = "CLICKHOUSE_URL", default_value = "http://localhost:8123")]
    pub clickhouse_url: String,

    #[clap(long, short = 'c')]
    pub config: Option<PathBuf>,
}
