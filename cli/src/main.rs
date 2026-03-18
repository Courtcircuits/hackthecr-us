use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{
    actions::{meals::{MealsAction, MealsActionResult}, restaurants::RestaurantsAction}, client::HTCClient, config::Config, crous::CrousRegion,
};

pub mod actions;
pub mod client;
pub mod config;
pub mod crous;

#[derive(Parser, Debug)]
#[clap(
    name = "crousctl",
    version,
    about = "crousctl controls the HackTheCrous scraping orchestra"
)]
struct Crousctl {
    #[clap(subcommand)]
    pub command: Command,
    #[clap(long, short = 'c')]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Subcommand, PartialEq, Eq, Hash)]
pub enum Command {
    Status,
    Restaurants {
        #[clap(long, short = 't')]
        target: CrousRegion,
        #[clap(long, short = 'd')]
        dry_run: bool,
    },
    Meals {
        #[clap(long, short = 't')]
        target: CrousRegion,
        #[clap(long, short = 'd')]
        dry_run: bool,
    },
    Schools {
        #[clap(long, short = 't')]
        target: String,
        #[clap(long, short = 'd')]
        dry_run: bool,
    },
    Schedule {},
    Generate {
        #[clap(long, short = 'u')]
        user: String,
        #[clap(long, short = 'd')]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() {
    let args = Crousctl::parse();

    let config_path = args.config.unwrap_or_else(|| {
        let home = std::env::var("HOME").expect("HOME env var not set");
        PathBuf::from(home).join(".config/htc.yml")
    });

    let Ok(config) = Config::from(&config_path) else {
        match args.command {
            Command::Generate { user, dry_run } => {
                let new_config = Config::generate(&user).expect("Couldn't generate config");
                if dry_run {
                    println!("{}", new_config.as_yaml().expect("Couldn't generate yaml"));
                } else {
                    new_config
                        .write(&config_path)
                        .expect("Couldn't write config");
                    println!(
                        "Configuration wrote at : {}",
                        config_path.to_str().unwrap()
                    );
                }
                return;
            }
            _ => {
                eprintln!("Config not found");
                panic!("Config not found");
            }
        }
    };

    let client = HTCClient::new(config.server, config.client_key_data, config.user);

    match args.command {
        Command::Status => {
            println!("Crousctl is running and ready to execute commands.");
        }
        Command::Restaurants { target, dry_run } => {
            let action = RestaurantsAction::new(target, dry_run, client);
            match action.execute().await {
                Ok(()) => {
                    println!("Successfully collected and stored restaurant data.");
                }
                Err(e) => {
                    eprintln!("Failed to collect restaurant data: {}", e);
                }
            }
        }
        Command::Meals { target, dry_run } => {
            let action = MealsAction::new(target, dry_run, client);
            match action.execute().await {
                Ok(()) => {
                    println!("Successfully collected and stored restaurant data.");
                }
                Err(e) => {
                    eprintln!("Failed to collect restaurant data: {}", e);
                }
            }
        }
        Command::Schools { target, dry_run } => {
            println!(
                "Schools command is not implemented yet. Target: {}, Dry run: {}",
                target, dry_run
            );
        }
        Command::Schedule {} => {
            todo!("Implement scheduling")
        }
        Command::Generate { user, dry_run } => {}
    }
}
