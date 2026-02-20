use std::path::{PathBuf};

use clap::{Parser, Subcommand};

use crate::crous::get_urls;


pub mod actions;
pub mod crous;

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
        target: String,
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

fn main() {
    let res = get_urls();
}
