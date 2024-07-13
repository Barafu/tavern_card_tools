
use crate::{Lorebook, LoreBookItem, BayaCharacter};

use std::convert::From;

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

impl From<&BayaCharacter> for TavernCardV2 {
    fn from(character: &BayaCharacter) -> Self {
        let mut new_character = TavernCardV2::new();
        let card_data = &mut new_character.data;

        let transfer_string = |s: &Option<String> | {
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
        };

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
        new_entry.keys = lorebook_entry.key
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