use std::fmt::Display;

use anyhow::{bail, Result};
use base64::prelude::*;
use bytes::Bytes;
use textwrap::{fill, Options};

use crate::tools;

pub const TEXT_KEY_PNG: &str = "Chara";

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq)]
pub struct CharacterBook {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scan_depth: Option<u32>,
    pub token_budget: Option<u32>,
    pub recursive_scanning: Option<bool>,
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    pub entries: Vec<CharacterBookEntry>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq)]
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq)]
pub struct TavernCardV2 {
    pub spec: Option<String>,
    pub spec_version: Option<String>,
    pub data: CharacterData,
    #[serde(skip)]
    pub image_data: Option<Bytes>, // For keeping PNG image along
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq)]
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
    pub alternate_greetings: Option<Vec<String>>,
    pub character_book: Option<CharacterBook>,
    pub tags: Option<Vec<String>>,
    pub creator: Option<String>,
    pub character_version: Option<String>,
    pub extensions:
        Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl TavernCardV2 {
    pub fn new() -> Self {
        let mut s = TavernCardV2::default();
        s.improve_card();
        s
    }

    /// Writes card into image
    ///
    /// Makes a copy of PNG image, with card tag added to it.
    pub fn into_png_image(&self) -> Result<Bytes> {
        let json_string = serde_json::to_string(self)?;
        let base64_json_string = BASE64_STANDARD.encode(json_string);
        let temp_image_holder;
        let image_data;
        match &self.image_data {
            Some(img) => image_data = img,
            None => {
                temp_image_holder = tools::get_default_image();
                image_data = &temp_image_holder;
            }
        }
        let edited_card = tools::write_text_to_png(
            TEXT_KEY_PNG,
            &base64_json_string,
            image_data,
        )?;
        Ok(edited_card)
    }

    pub fn from_png_image(image_data: &Bytes) -> Result<Self> {
        let raw_text = tools::read_text_chunk(image_data, TEXT_KEY_PNG)?;
        if raw_text.is_none() {
            bail!("No {} entry in PNG tEXt chunks", TEXT_KEY_PNG);
        };
        let text = BASE64_STANDARD.decode(raw_text.unwrap())?;
        if !text.starts_with(&[b'{']) {
            bail!(
                "{} entry in PNG tEXt chunks does not start with '{{'",
                TEXT_KEY_PNG
            );
        }
        // Try to convert tag into tavern card data
        let mut card = serde_json::from_slice::<TavernCardV2>(&text);
        if card.is_err() {
            // Sometimes the tag contains only the data portion
            match serde_json::from_slice::<CharacterData>(&text) {
                Ok(card_data) => {
                    card = Ok(TavernCardV2 {
                        data: card_data,
                        ..Default::default()
                    });
                }
                Err(e) => {
                    bail!(
                        "Failed to parse {} entry in PNG tEXt chunks: {}",
                        TEXT_KEY_PNG,
                        e
                    );
                }
            }
        }
        let mut card = card.unwrap();
        card.image_data = Some(image_data.clone());
        Ok(card)
    }

    /// Make changes to better conform the specification
    fn improve_card(&mut self) {
        if self.spec.is_none() {
            self.spec = Some("chara_card_v2".to_string());
        }
        if self.spec_version.is_none() {
            self.spec_version = Some("2.0".to_string());
        }
        if self.data.name.is_none() {
            self.data.name = Some("".to_string());
        }
        if self.data.description.is_none() {
            self.data.description = Some("NO NAME".to_string());
        }
    }
}

impl Display for TavernCardV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Turns &Option<String> into &str
        fn no_opt(o: &Option<String>) -> &str {
            const NONE_STR: &str = "NONE";
            o.as_ref().map(|x| x.as_str()).unwrap_or(NONE_STR)
        }

        let mut lines = vec![
            ("Character name:", no_opt(&self.data.name)),
            ("Description:", no_opt(&self.data.description)),
            ("Personality:", no_opt(&self.data.personality)),
            ("Scenario:", no_opt(&self.data.scenario)),
            ("First message:", no_opt(&self.data.first_mes)),
            ("Dialog example:", no_opt(&self.data.mes_example)),
            ("Creator notes:", no_opt(&self.data.creator_notes)),
            ("System prompt:", no_opt(&self.data.system_prompt)),
            (
                "Post history instructions:",
                no_opt(&self.data.post_history_instructions),
            ),
            ("Creator:", no_opt(&self.data.creator)),
        ];

        // Print alternative greetings, if present
        let ag_store;
        if let Some(alternative_greetings) = &self.data.alternate_greetings {
            ag_store = alternative_greetings.join("\n\n====\n\n");
            lines.push(("Alternative greetings:", &ag_store));
        }

        // Print lorebook
        let lb_store;
        if let Some(character_book) = &self.data.character_book {
            lb_store = character_book
                .entries
                .iter()
                .map(|x| (x.keys.join(","), &x.content))
                .map(|x| {
                    if x.0.is_empty() {
                        ("NO KEYS".to_string(), x.1)
                    } else {
                        x
                    }
                })
                .map(|x| format!("{} : {}", x.0, x.1))
                .collect::<Vec<String>>()
                .join("\n");
            lines.push(("Lorebook:", &lb_store));
        }

        // Now to convert the lines vector into a pretty string
        let mut output = String::new();
        let tw = *[textwrap::termwidth(), 80usize].iter().min().unwrap();
        let options =
            Options::new(tw).initial_indent("").subsequent_indent("    ");
        for (key, value) in lines {
            let mut line = format!("{}: {}\n", key, value);
            line = fill(&line, &options);
            output += &line;
        }
        write!(f, "{}", output)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Result;

    #[allow(unused_imports)]
    use tools;

    fn create_test_card() -> TavernCardV2 {
        let mut card = TavernCardV2::new();
        card.data.name = Some(String::from("Test name"));
        card.data.description = Some(String::from("Test description"));
        card.data.personality = Some(String::from("Test personality"));
        card.data.scenario = Some(String::from("Test scenario"));
        card.data.first_mes = Some(String::from("Test first message"));
        card.data.mes_example = Some(String::from("Test dialog example"));
        card.data.character_book = Some(CharacterBook::default());
        let mut entry1 = CharacterBookEntry::default();
        entry1.content = String::from("Test book entry 1");

        let mut entry2 = CharacterBookEntry::default();
        entry2.content = String::from("Test book entry 2");

        card.data.character_book.as_mut().unwrap().entries.push(entry1);
        card.data.character_book.as_mut().unwrap().entries.push(entry2);
        card.image_data = Some(tools::get_default_image());
        let image_with_tag = card.into_png_image().unwrap();
        card.image_data = Some(image_with_tag);
        card
    }

    #[test]
    fn test_equal_sanity() {
        let card1 = create_test_card();
        let card2 = create_test_card();
        assert_eq!(card1, card2);
    }

    #[test]
    fn test_write_and_read() -> Result<()> {
        let card = create_test_card();
        let image = card.into_png_image()?;
        let card2 = TavernCardV2::from_png_image(&image)?;
        assert_eq!(card, card2);
        // tools::write_image_to_file(&image, &std::path::Path::new("testing/test_card.png"))?;
        Ok(())
    }
}
