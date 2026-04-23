use std::{path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use color_print::cprintln;
use htc::{client::HTCClient, config::{Config, CronConfig, EntityScheduleConfig, OutputFormat}, regions::CrousRegion};

use crate::{
    actions::{
        Executable, meals::MealsAction, restaurants::RestaurantsAction, schedule::ScheduleAction,
    },
};

pub mod actions;

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
        #[clap(long, short = 'n')]
        server_name: Option<String>,
        #[clap(long, short = 'd')]
        dry_run: bool,
        #[clap(long, short = 'o', default_value = "yaml")]
        output: Option<OutputFormat>,
        #[clap(long, short = 's')]
        secret_name: Option<String>,
        #[clap(long)]
        restaurants_schedule: Option<String>,
        #[clap(long, num_args = 1..)]
        restaurants_targets: Vec<String>,
        #[clap(long)]
        meals_schedule: Option<String>,
        #[clap(long, num_args = 1..)]
        meals_targets: Vec<String>,
        #[clap(long)]
        schools_schedule: Option<String>,
        #[clap(long, num_args = 1..)]
        schools_targets: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    let args = Crousctl::parse();

    let config_path = args.config.unwrap_or_else(|| {
        let home = std::env::var("HOME").expect("HOME env var not set");
        PathBuf::from(home).join(".config/htc.yml")
    });

    if let Command::Generate {
        user,
        server_name,
        dry_run,
        output,
        secret_name,
        restaurants_schedule,
        restaurants_targets,
        meals_schedule,
        meals_targets,
        schools_schedule,
        schools_targets,
    } = args.command
    {
        let build_entity = |schedule: Option<String>, targets: Vec<String>| {
            schedule.map(|s| EntityScheduleConfig { schedule: s, target: targets })
        };
        let restaurants = build_entity(restaurants_schedule, restaurants_targets);
        let meals = build_entity(meals_schedule, meals_targets);
        let schools = build_entity(schools_schedule, schools_targets);
        let schedule = if restaurants.is_some() || meals.is_some() || schools.is_some() {
            Some(CronConfig { restaurants, meals, schools })
        } else {
            None
        };

        let new_config = Config::generate(&user, server_name.as_deref(), schedule).expect("Couldn't generate config");
        if dry_run {
            new_config
                .print(output.unwrap_or(OutputFormat::Yaml), secret_name.as_deref())
                .expect("Couldn't format config");
        } else {
            new_config
                .write(
                    &config_path,
                    output.unwrap_or(OutputFormat::Yaml),
                    secret_name.as_deref(),
                )
                .expect("Couldn't write config");
        }
        return;
    }

    let Ok(config) = Config::from(&config_path) else {
        cprintln!("💣 <red>Config not found</red>");
        exit(0)
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
                let schedule = ScheduleAction::try_from_config(config, client)
                    .map_err(|e| {
                        cprintln!("💣 <red>{}</red>", e.to_string());
                    })
                    .unwrap();
                let _ = schedule.schedule().await;
            }
            None => {
                cprintln!("💣 <red>No schedule config</red>");
            }
        },
        Command::Generate { .. } => unreachable!(),
    }
}
