use anyhow::Result;
use base64::prelude::*;
use bytes::Bytes;

use crate::tools;

#[derive(serde::Serialize, Debug, Default)]
pub struct CharacterBook {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scan_depth: Option<u32>,
    pub token_budget: Option<u32>,
    pub recursive_scanning: Option<bool>,
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    pub entries: Vec<CharacterBookEntry>,
}

#[derive(serde::Serialize, Debug, Default)]
pub struct CharacterBookEntry {
    pub keys: Vec<String>,
    pub content: String,
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    pub enabled: bool,
    pub insertion_order: u32,
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

#[derive(serde::Serialize, Debug, Default)]
pub struct TavernCardV2 {
    pub spec: String,
    pub spec_version: String,
    pub data: CharacterData,
}

#[derive(serde::Serialize, Debug, Default)]
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
        }
    }
}

/// Write Tavern tag into the image.
pub fn write_tavern_card(char: &TavernCardV2, image_data: &Bytes) -> Result<Bytes> {
    let json_string = serde_json::to_string(char)?;
    let base64_json_string = BASE64_STANDARD.encode(json_string);
    let edited_card = tools::write_text_to_png("Chara", &base64_json_string, image_data)?;
    Ok(edited_card)
}
