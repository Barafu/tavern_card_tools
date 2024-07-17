#![allow(dead_code)]

use std::path::Path;

mod baya_download;
mod tavern_card_v2;
mod tools;
mod deasterisk;
//mod example;

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

    let args: Vec<String> = std::env::args().collect();
    let args_r: Vec<&str> = args.iter().map(|x| x.as_str()).collect();

    // Run action according to the first CLI arg, or print usage.
    let mut error_flag = Ok(());
    match args_r.as_slice() {
        [_, "baya_get", url] => {
            error_flag = baya_download::download_card_from_baya_url(*url);
        },
        [_, "de8", path] => {
            let p = Path::new(*path);
            error_flag = deasterisk::deasterisk_tavern_file(p);
        }
        _ => wrong_usage(),
    }

    if let Err(e) = error_flag {
        eprintln!("ERROR: {:?} \nABORT", e);
        std::process::exit(1);
    }
}

fn wrong_usage() {
    eprintln!("Wrong arguments!");
    //dbg!(env::args().collect::<Vec<String>>());
    print_usage();
    std::process::exit(2);
}

// In future this will print the user help.
fn print_usage() {
    println!("Usage: baya_get <url>");
}
