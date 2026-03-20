use std::{path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use color_print::cprintln;
use htc::regions::CrousRegion;

use crate::{
    actions::{
        Executable, meals::MealsAction, restaurants::RestaurantsAction, schedule::ScheduleAction,
    },
    client::HTCClient,
    config::Config,
};

pub mod actions;
pub mod client;
pub mod config;

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
                    println!("Configuration wrote at : {}", config_path.to_str().unwrap());
                }
                return;
            }
            _ => {
                cprintln!("💣 <red>Config not found</red>");
                exit(0)
            }
        }
    };

    let cron_config = config.schedule;
    let client = HTCClient::new(config.server, config.client_key_data, config.user);

    match args.command {
        Command::Status => {
            println!("Crousctl is running and ready to execute commands.");
        }
        Command::Restaurants { target, dry_run } => {
            let action = RestaurantsAction::new(target, dry_run, client);
            match action.execute().await {
                Ok(()) => {
                    cprintln!(
                        "✅ <green>Successfully collected and stored restaurant data.</green>"
                    );
                }
                Err(e) => {
                    cprintln!("💣 <red>Failed to collect restaurant data: {}</red>", e);
                }
            }
        }
        Command::Meals { target, dry_run } => {
            let action = MealsAction::new(target, dry_run, client);
            match action.execute().await {
                Ok(()) => {
                    cprintln!(
                        "✅ <green>Successfully collected and stored restaurant data.</green>"
                    );
                }
                Err(e) => {
                    cprintln!("💣 <red>Failed to collect restaurant data: {}</red>", e);
                }
            }
        }
        Command::Schools { target, dry_run } => {
            println!(
                "Schools command is not implemented yet. Target: {}, Dry run: {}",
                target, dry_run
            );
        }
        Command::Schedule {} => match cron_config {
            Some(config) => {
                let schedule = ScheduleAction::try_from_config(config, client).map_err(|e| {
                    cprintln!("💣 <red>{}</red>", e.to_string());
                }).unwrap();
                let _ = schedule.schedule().await;
            }
            None => {
                cprintln!("💣 <red>No schedule config</red>");
            }
        },
        Command::Generate {
            user: _user,
            dry_run: _dry_run,
        } => {}
    }
}
