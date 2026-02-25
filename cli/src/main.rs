use std::path::{PathBuf};

use clap::{Parser, Subcommand};

use crate::{actions::restaurants::RestaurantsAction, crous::CrousRegion};


pub mod actions;
pub mod crous;
pub mod client;

#[derive(Parser, Debug)]
#[clap(name = "crousctl", version, about = "crousctl controls the HackTheCrous scraping orchestra")]
struct Crousctl {
    #[clap(subcommand)]
    pub command: Command
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
        target: String,
        #[clap(long, short = 'd')]
        dry_run: bool,
    },
    Schools {
        #[clap(long, short = 't')]
        target: String,
        #[clap(long, short = 'd')]
        dry_run: bool,
    },
    Schedule {
        #[clap(long, short = 'c')]
        config: PathBuf
    }
}

#[tokio::main]
async fn main() {
    let args = Crousctl::parse();

    match args.command {
        Command::Status => {
            println!("Crousctl is running and ready to execute commands.");
        }
        Command::Restaurants { target, dry_run } => {
            let action = RestaurantsAction::new(target, dry_run);
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
            println!("Meals command is not implemented yet. Target: {}, Dry run: {}", target, dry_run);
        }
        Command::Schools { target, dry_run } => {
            println!("Schools command is not implemented yet. Target: {}, Dry run: {}", target, dry_run);
        }
        Command::Schedule { config } => {
            println!("Schedule command is not implemented yet. Config path: {:?}", config);
        }
    }   
}
