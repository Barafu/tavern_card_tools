#![allow(dead_code)]

use anyhow::Result;
use clap::{Parser, ValueHint};
use std::path::{Path, PathBuf};

mod actions;
mod baya_download;
mod deasterisk;
mod tavern_card_v2;
mod tools;
//mod example;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(author = "Barafu Albino <barafu_develops@albino.email",
     version = APP_VERSION,
     about = "Tools for tavern cards", long_about = None)]
#[group(multiple = false)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// If no command is provided, "print" command is used by default.
    card_path: Option<String>,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Download tavern card from BackyardAI
    #[command(name = "baya_get")]
    #[command(arg_required_else_help = true)]
    BayaGet {
        /// URL at Backyard AI website to download from
        #[arg()]
        url: String,
    },
    /// Remove paired asterisks from text in tavern card. Makes a copy of the image and renames it to de8.<old_name.png>
    #[command(arg_required_else_help = true)]
    De8 {
        /// Path to image.png
        #[arg(value_hint = ValueHint::FilePath)]
        path: PathBuf,

        /// Overwrite output file if it exists already
        #[arg(long)]
        force: bool,
    },
    /// Print the content of the card
    #[command(arg_required_else_help = true)]
    Print {
        /// Path to image.png
        #[arg(value_hint = ValueHint::FilePath)]
        path: PathBuf,
    },
    /// Print the JSON of the card
    #[command(name = "print_all")]
    #[command(arg_required_else_help = true)]
    PrintJson {
        /// Path to image.png
        #[arg(value_hint = ValueHint::FilePath)]
        path: PathBuf,
    },
}

fn main() {
    // Prepare debug logging.
    #[cfg(debug_assertions)]
    {
        use env_logger::Builder;
        use std::fs::File;
        let target = Box::new(
            File::create("testing/last_run.log").expect("Can't create file"),
        );

        Builder::new()
            .target(env_logger::Target::Pipe(target))
            .filter(None, log::LevelFilter::Info)
            .init();
    }

    // Print intro
    println!("tavern card tools v{}", APP_VERSION);

    if let Err(err) = parse_args() {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}

fn parse_args() -> Result<()> {
    let args = Cli::parse();

    if args.card_path.is_none() && args.command.is_none() {
        eprintln!("Error: No command given");
        // println!("{}", Cli::);
        std::process::exit(1);
    }

    if let Some(card_path) = args.card_path {
        actions::print_tavern_card_from_path(Path::new(&card_path))?;
        return Ok(());
    }

    match args.command.unwrap() {
        Commands::BayaGet { url } => {
            baya_download::download_card_from_baya_url(&url)?
        }
        Commands::De8 { path, force } => {
            deasterisk::deasterisk_tavern_file(&path, force)?
        }
        Commands::Print { path } => {
            actions::print_tavern_card_from_path(&path)?
        }
        Commands::PrintJson { path } => actions::print_json_from_path(&path)?,
    };
    Ok(())
}
