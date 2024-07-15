//! Tools to download a character from Backyard AI

use std::{
    io::{self, Write},
    path::PathBuf,
    thread,
    time::Duration,
};

use crate::{
    tavern_card_v2::*,
    tools::{self, write_image_to_file},
};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use log::info;
use soup::prelude::*;

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
    Lorebook: Option<Lorebook>,
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

pub fn download_card_from_baya_url(url: &str) -> Result<()> {
    // Forcibly flush stdout before blocking operations, otherwise the line before long operations does not display.
    let flush = || io::stdout().flush().unwrap();

    print!("Downloading web page: ");
    flush();
    let body = tools::download_page(url)?;
    println!("Done!");

    print!("Parsing downloaded page: ");
    let baya_character = parse_page(&body).context("Could not parse character JSON")?;
    println!("Done!");

    let display_char_name: String = baya_character
        .aiDisplayName
        .clone()
        .unwrap_or_else(|| "NO_NAME_SET".to_string());
    println!("Character name is: {}", display_char_name);

    info!("\nCHARACTER INFO:");
    info!("{:#?}", &baya_character);

    // Download the image, if it is linked on the page. Otherwise, use default image.
    let card_name = PathBuf::from(format!("{}.png", display_char_name));
    let mut card_image;
    if !baya_character.Images.is_empty() {
        // Download the first image linked on card.
        let url = &baya_character.Images[0].imageUrl;
        print!("Downloading image: ");
        flush();
        card_image = tools::download_image(url)?;
        card_image = tools::convert_to_png(&card_image)?;
    } else {
        print!("No image provided, using default image.");
        card_image = tools::get_default_image();
    }
    println!("Done!");

    print!("Writing tavern card: ");
    flush();
    let tavern_card = TavernCardV2::from(&baya_character);
    let tavern_image =
        write_tavern_card(&tavern_card, &card_image).context("Could not write tavern card")?;
    write_image_to_file(&tavern_image, &card_name)? ;
    println!("Done!");
    print!("Fap away!");
    flush();
    thread::sleep(Duration::from_millis(150));
    println!("\rAll done!");
    flush();
    Ok(())
}

/// Extracts character data from the downloaded web page.
fn parse_page(body: &str) -> Result<BayaCharacter> {
    let soup = soup::Soup::new(body);
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
            bail!(
                "Could not parse character JSON: {:?}  Error path: {:?}",
                e,
                err_path
            );
        }
    }
}

impl From<&BayaCharacter> for TavernCardV2 {
    fn from(character: &BayaCharacter) -> Self {
        let mut new_character = TavernCardV2::new();
        let card_data = &mut new_character.data;

        let transfer_string = |s: &Option<String>| {
            let sn = s.clone();
            sn.filter(|x| !x.is_empty())
        };

        card_data.name = transfer_string(&character.aiDisplayName);
        card_data.description = transfer_string(&character.aiPersona);
        card_data.scenario = transfer_string(&character.scenario);
        card_data.first_mes = transfer_string(&character.firstMessage);
        card_data.mes_example = transfer_string(&character.customDialogue);
        card_data.creator_notes = transfer_string(&character.authorNotes);
        card_data.system_prompt = transfer_string(&character.basePrompt);
        card_data.personality = transfer_string(&character.description);

        for tag in &character.Tags {
            card_data.tags.push(tag.name.clone());
        }

        let author_name = match &character.Author {
            Some(author) => author.username.clone(),
            None => "".to_string(),
        };
        card_data.creator = transfer_string(&Some(author_name));

        //Now copy the lorebook
        if let Some(lorebook) = &character.Lorebook {
            if !lorebook.LorebookItems.is_empty() {
                card_data.character_book = Some(lorebook.into());
            }
        }

        new_character
    }
}

impl From<&LoreBookItem> for CharacterBookEntry {
    fn from(lorebook_entry: &LoreBookItem) -> Self {
        let mut new_entry = CharacterBookEntry::default();
        new_entry.keys = lorebook_entry
            .key
            .split(",")
            .map(|x| x.trim().to_string())
            .collect();
        new_entry.content = lorebook_entry.value.clone();
        new_entry
    }
}

impl From<&Lorebook> for CharacterBook {
    fn from(lorebook: &Lorebook) -> Self {
        let mut new_book = CharacterBook::default();
        for lorebook_entry in &lorebook.LorebookItems {
            new_book.entries.push(lorebook_entry.into());
        }
        new_book
    }
}


