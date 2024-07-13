#![allow(dead_code)]

use anyhow::{bail, Context, Result };
use base64::prelude::*;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use env_logger::Builder;
use image::ImageFormat;
use log::info;
use serde_json;
use soup::prelude::*;
use std::io::{BufWriter, Cursor};
use std::{
    env,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

mod tavern_card_v2;
use tavern_card_v2::TavernCardV2;

#[allow(non_snake_case, dead_code)]
#[derive(serde::Deserialize, Debug)]
pub struct BayaCharacter {
    aiName: Option<String>,
    aiDisplayName: Option<String>,
    description: Option<String>,
    authorNotes: Option<String>,
    createdAt: DateTime<Utc>,
    updatedAt: DateTime<Utc>,
    aiPersona: Option<String>,
    basePrompt: Option<String>,
    customDialogue: Option<String>,
    firstMessage: Option<String>,
    scenario: Option<String>,
    temperature: Option<f32>,
    repeatLastN: Option<i32>,
    repeatPenalty: Option<f32>,
    isNsfw: Option<bool>,
    grammar: Option<String>,
    topP: Option<f32>,
    minP: Option<f32>,
    minPEnabled: Option<bool>,
    topK: Option<i32>,
    promptTemplate: Option<String>,
    Author: Option<Author>,
    ModelFamily: Option<ModelFamily>,
    Tags: Vec<Tag>,
    Images: Vec<Image>,
    Lorebook  : Option<Lorebook>,
}

#[allow(non_snake_case, dead_code)]
#[derive(serde::Deserialize, Debug)]
struct Lorebook {
    LorebookItems: Vec<LoreBookItem>,
}

#[allow(non_snake_case, dead_code)]
#[derive(serde::Deserialize, Debug)]
struct LoreBookItem {
    key: String,
    order: String,
    value: String,
}

#[allow(non_snake_case, dead_code)]
#[derive(serde::Deserialize, Debug)]
struct Image {
    imageUrl: String,
    label: Option<String>,
}

#[allow(non_snake_case, dead_code)]
#[derive(serde::Deserialize, Debug)]
struct Tag {
    name: String,
}

#[allow(non_snake_case, dead_code)]
#[derive(serde::Deserialize, Debug)]
struct Author {
    username: String,
}

#[allow(non_snake_case, dead_code)]
#[derive(serde::Deserialize, Debug)]
struct ModelFamily {
    displayName: String,
    promptFormat: String,
}

fn main() {
    // Prepare debug logging.
    #[cfg(debug_assertions)]
    {
        let target = Box::new(File::create("last_run.log").expect("Can't create file"));

        Builder::new()        
            .target(env_logger::Target::Pipe(target))
            .filter(None, log::LevelFilter::Info)
            .init();
    }

    // Print intro
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("tavern card tools v{}", VERSION);

    let args: Vec<String> = env::args().collect();
    let args_r: Vec<&str> = args.iter().map(|x| x.as_str()).collect();

    // Run action according to the first CLI arg, or print usage.
    let mut error_flag = Ok(());
    match args_r.as_slice() {
        [_, "baya_get", url] => {
            error_flag = download_card_from_url(*url);
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
    dbg!(env::args().collect::<Vec<String>>());
    print_usage();
    std::process::exit(2);
}

// In future this will print the user help.
fn print_usage() {
    println!("Usage: baya_get <url>");
}

fn download_card_from_url(url: &str) -> Result<()> {
    // Forcibly flush stdout before blocking operations, otherwise the line before long operations does not display.
    let flush = || io::stdout().flush().unwrap();

    print!("Downloading web page: ");
    flush();
    let body = download_page(url)?;
    println!("Done!");

    print!("Parsing downloaded page: ");
    let char = parse_page(&body).context("Could not parse character JSON")?;
    let no_name_set_text: String = "NO_NAME_SET".to_string();
    println!("Done!");
    println!("Character name is: {}", char.aiDisplayName.as_ref().unwrap_or_else( || &no_name_set_text));
    info!("\nCHARACTER INFO:");
    info!("{:#?}", &char);

    // Download the image, if it is linked on the page. Otherwise, use default image.
    let image_name = PathBuf::from(format!("{}.png", char.aiDisplayName.as_ref().unwrap_or_else( || &no_name_set_text)));
    if !char.Images.is_empty() {
        // Download the first image linked on card.
        let url = &char.Images[0].imageUrl;
        print!("Downloading image: ");
        flush();
        download_image(&image_name, url)?;
    } else {
        print!("No image provided, using default image.");
        store_default_image(&image_name)?;
    }
    println!("Done!");

    print!("Writing tavern card: ");
    flush();
    write_tavern_card(&char, &image_name).context("Could not write tavern card")?;
    println!("Done!");
    print!("Fap away!");
    flush();
    thread::sleep(Duration::from_millis(150));
    println!("\rAll done!");
    flush();
    Ok(())
}

/// Downloads web page by URL, returns contents
fn download_page(url: &str) -> Result<String> {
    let response = reqwest::blocking::get(url)?;
    if response.status().is_success() {
        let body = response.text()?;
        return Ok(body);
    } else {
        bail!("Failed to download the web page: {:?}", response.status());
    }
}

/// Extracts character data from the downloaded web page.
fn parse_page(body: &str) -> Result<BayaCharacter> {
    let soup = Soup::new(body);
    let scr = soup
        .tag("script")
        .attr("id", "__NEXT_DATA__")
        .find()
        .context("Did not find __NEXT_DATA__")?;

    let scr_text = scr.text();

    info!("SCRIPT DATA:");
    info!("{}", &scr_text);

    let mut json: serde_json::Value =
        serde_json::from_str(&scr.text()).context("JSON was not well-formatted")?;

    let pointer = "/props/pageProps/trpcState/json/queries/0/state/data/character";
    let char_json = json
        .pointer_mut(pointer)
        .context("Could not find character block")?;

    let json_string = serde_json::to_string_pretty(&char_json)?;
    info!("CHAR JSON:");
    info!("{:#?}", &json_string);

    let ds = &mut serde_json::Deserializer::from_str(&json_string);
    let result: Result<BayaCharacter, _> = serde_path_to_error::deserialize(ds);
    match result {
        Ok(bc) => Ok(bc),
        Err(e) => {
            let err_path = e.path().to_string();
            bail!("Could not parse character JSON: {:?}  Error path: {:?}", e, err_path );
        }
    }
}

/// Download image from URL if the URL is present in character JSON. If not, or if download fails,
/// use builtin default image.
fn download_image(image_path: &Path, url: &str) -> Result<()> {
    let downloaded_data;
    // Try to download the image.
    let response = reqwest::blocking::get(url).context("No response when downloading image!")?;
    if response.status().is_success() {
        downloaded_data = response
            .bytes()
            .context("Could not read the downloaded data.")?;
    } else {
        bail!("Could not download image: status {:?}", response.status());
    };

    // Convert to PNG if it's not already.
    let downloaded_image =
        convert_to_png(&downloaded_data).context("Could not convert image to PNG")?;

    // Write the image to disk.
    let mut file = File::create(image_path)?;
    file.write_all(&downloaded_image)?;

    Ok(())
}

fn convert_to_png(image_data: &Bytes) -> Result<Bytes> {
    // Try to determine the format of the input image
    let format = image::guess_format(image_data.as_ref())?;
    // If it's already a PNG, return the original data
    if format == ImageFormat::Png {
        return Ok(image_data.clone());
    }
    // If it's not a PNG, decode the image
    let img = image::load_from_memory(image_data.as_ref())?;
    // Prepare a buffer to store the PNG output
    let mut png_buffer = Vec::new();
    // Convert the image to PNG and write it to the buffer
    img.write_to(&mut Cursor::new(&mut png_buffer), ImageFormat::Png)?;
    let png_output: Bytes = Bytes::from(png_buffer);
    Ok(png_output)
}

/// Store default image to disk and pretend that it was downloaded.
fn store_default_image(image_path: &Path) -> Result<()> {
    const DEFAULT_IMAGE: Bytes = Bytes::from_static(include_bytes!("no_face.png"));
    let mut file = File::create(image_path)?;
    file.write_all(&DEFAULT_IMAGE)?;
    Ok(())
}

/// Reopens the image. Writes Tavern tag into the image.
fn write_tavern_card(char: &BayaCharacter, image_path: &Path) -> Result<()> {
    let card = TavernCardV2::from(char);
    let json_string = serde_json::to_string(&card)?;
    // dbg!(&json_string);
    let base64_json_string = BASE64_STANDARD.encode(json_string);
    write_text_to_png("Chara", &base64_json_string, image_path)?;

    Ok(())
}

fn write_text_to_png(key: &str, value: &str, image_path: &Path) -> Result<()> {
    // # Decode
    // The decoder is a build for reader and can be used to set various decoding options
    // via `Transformations`. The default output transformation is `Transformations::IDENTITY`.
    let decoder = png::Decoder::new(File::open(image_path)?);
    let mut reader = decoder.read_info()?;
    // Allocate the output buffer.
    let png_info = reader.info().clone();
    let mut buf = vec![0; reader.output_buffer_size()];
    // Save picture to memory.
    let mut memory: Vec<Vec<u8>> = Vec::new();
    while let Ok(info) = reader.next_frame(&mut buf) {
        let mut frame = buf.clone();
        frame.resize(info.buffer_size(), 0);
        //let bytes = Bytes::&buf[..info.buffer_size()].;
        memory.push(frame);
    }
    drop(reader);

    // # Encode
    // let path_out = image_path.with_file_name("output2");
    let file = File::create(image_path)?;
    let ref mut w = BufWriter::new(file);

    // Get defaults for interlaced parameter.
    let mut info_out = png_info.clone();
    let info_default = png::Info::default();

    // Edit previous info
    info_out.interlaced = info_default.interlaced;
    let mut encoder = png::Encoder::with_info(w, info_out)?;
    encoder.set_depth(png_info.bit_depth);

    // Edit some attribute
    encoder.add_text_chunk(key.to_string(), value.to_string())?;

    // Save picture with changed info
    let mut writer = encoder.write_header()?;
    for frame in memory {
        writer.write_image_data(&frame)?;
    }
    Ok(())
}
