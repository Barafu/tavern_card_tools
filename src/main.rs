#![allow(dead_code)]

use std::path::Path;
use anyhow::Result;
use clap::Parser;

mod baya_download;
mod tavern_card_v2;
mod tools;
mod deasterisk;
//mod example;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Download tavern card from BackyardAI
    #[command(name = "baya_get", author, version, about, long_about = None)]
    #[command(arg_required_else_help = true)]
    BayaGet {
        /// URL at Backyard AI website to download from
        #[arg(short, long)]
        url: String,
    },
    /// Remove paired asterisks from text in tavern card. Makes a copy of the image and renames it to de8.old_name.png
    #[command(author, version, about, long_about = None)]
    #[command(arg_required_else_help = true)]
    De8 {
        /// Path to image.png
        #[arg(short, long)]
        path: String,
    },
}

fn main() {
    // Prepare debug logging.
    #[cfg(debug_assertions)]
    {
        use env_logger::Builder;
        use std::fs::File;
        let target = Box::new(File::create("testing/last_run.log")
            .expect("Can't create file"));

        Builder::new()
            .target(env_logger::Target::Pipe(target))
            .filter(None, log::LevelFilter::Info)
            .init();
    }

    // Print intro
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("tavern card tools v{}", VERSION);

    if let Err(err) = parse_args() {
        println!("Error: {}", err);
        std::process::exit(1);
    }

}

fn parse_args() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::BayaGet { url } => baya_download::download_card_from_baya_url(&url)?,
        Commands::De8 { path } => deasterisk::deasterisk_tavern_file(Path::new(&path))?,
    };
    Ok(())

}
