use anyhow::{bail, Result};
use base64::prelude::*;
use bytes::Bytes;

use crate::tools;

const TEXT_KEY_PNG: &str = "Chara";

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct CharacterBook {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scan_depth: Option<u32>,
    pub token_budget: Option<u32>,
    pub recursive_scanning: Option<bool>,
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    pub entries: Vec<CharacterBookEntry>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct CharacterBookEntry {
    pub keys: Vec<String>,
    pub content: String,
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    pub enabled: bool,
    pub insertion_order: Option<u32>,
    pub case_sensitive: Option<bool>,
    pub name: Option<String>,
    pub priority: Option<u32>,
    pub id: Option<u32>,
    pub comment: Option<String>,
    pub selective: Option<bool>,
    pub secondary_keys: Option<Vec<String>>,
    pub constant: Option<bool>,
    pub position: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct TavernCardV2 {
    pub spec: String,
    pub spec_version: String,
    pub data: CharacterData,
    #[serde(skip)]
    pub image_data: Option<Bytes>, // For keeping PNG image along
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct CharacterData {
    pub name: Option<String>,
    pub description: Option<String>,
    pub personality: Option<String>,
    pub scenario: Option<String>,
    pub first_mes: Option<String>,
    pub mes_example: Option<String>,
    pub creator_notes: Option<String>,
    pub system_prompt: Option<String>,
    pub post_history_instructions: Option<String>,
    pub alternate_greetings: Vec<String>,
    pub character_book: Option<CharacterBook>,
    pub tags: Vec<String>,
    pub creator: Option<String>,
    pub character_version: Option<String>,
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
}

impl TavernCardV2 {
    pub fn new() -> Self {
        Self {
            spec: "chara_card_v2".to_string(),
            spec_version: "2.0".to_string(),
            data: CharacterData::default(),
            image_data: None,
        }
    }

    /// Writes card into image
    ///
    /// Makes a copy of PNG image, with card tag added to it.
    pub fn write_tavern_card(&self, image_data: &Bytes) -> Result<Bytes> {
        let json_string = serde_json::to_string(self)?;
        let base64_json_string = BASE64_STANDARD.encode(json_string);
        let edited_card = tools::write_text_to_png(TEXT_KEY_PNG, &base64_json_string, image_data)?;
        Ok(edited_card)
    }

    pub fn read_from_png(image_data: &Bytes) -> Result<Self> {
        let raw_text = tools::read_text_chunk(image_data, TEXT_KEY_PNG)?;
        if raw_text.is_none() {
            bail!("No {} entry in PNG tEXt chunks", TEXT_KEY_PNG);
        };
        let text = BASE64_STANDARD.decode(raw_text.unwrap())?;
        let mut card: TavernCardV2 = serde_json::from_slice(&text)?;
        card.image_data = Some(image_data.clone());
        Ok(card)
    }
}
