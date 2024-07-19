//! Functions to remove asterisks.

use std::path::Path;

use anyhow::{bail, Result};
use log::info;

use crate::{
    tavern_card_v2::TavernCardV2,
    tools::{self, read_image_from_file},
};

/// Remove asterisks from text
///
/// Removes paired asterisks (*) from text, but leaves adjacent asterisks
/// untouched. For example:
/// Original: Hello *world*, this is a **test** of *asterisks*.
/// Modified: Hello world, this is a **test** of asterisks.
fn remove_paired_asterisks(input_str: &str) -> String {
    let input: Vec<char> = input_str.chars().collect();
    const AST: char = '*';
    const BRK: char = '\n';
    let mut pos_to_elim: Vec<usize> = Vec::new();
    let mut detecting_pair = false;
    let mut pair_start_index: usize = 0;
    for (i, ch) in input.iter().enumerate() {
        if *ch == AST {
            // if a part of a group of asterisks - skip.
            if let Some(c) = input.get(i + 1) {
                if *c == AST {
                    continue;
                }
            }
            if let Some(c) = input.get(i.wrapping_sub(1)) {
                if *c == AST {
                    continue;
                }
            }
            // Start detecting a pair
            if !detecting_pair {
                detecting_pair = true;
                pair_start_index = i;
            }
            // Pair detected - mark both for removal.
            else {
                pos_to_elim.push(pair_start_index);
                pos_to_elim.push(i);
                detecting_pair = false;
            }
        }
        if *ch == BRK {
            detecting_pair = false;
        }
    }

    // Now we have positions of all chars to eliminate. Copy to output all remaining chars.
    let res: String = input
        .iter()
        .enumerate()
        .filter(|(i, _)| !pos_to_elim.contains(i))
        .map(|(_, v)| v)
        .collect();
    res
}

/// Removes asterisks from relevant fields of tavern card
pub fn deasterisk_tavern_card(tavern_card: &mut TavernCardV2) {
    let d = &mut tavern_card.data;
    let de8 = |x: &mut Option<String>| {
        let t = x.as_ref().map(|y| remove_paired_asterisks(&y));
        *x = t;
    };
    de8(&mut d.description);
    de8(&mut d.personality);
    de8(&mut d.scenario);
    de8(&mut d.first_mes);
    de8(&mut d.mes_example);
    if let Some(cb) = &mut d.character_book {
        for e in &mut cb.entries {
            e.content = remove_paired_asterisks(&e.content);
        }
    }
    if let Some(ag) = &mut d.alternate_greetings {
        for g in ag.iter_mut() {
            *g = remove_paired_asterisks(g);
        }
    }
}

// Opens file, applies deasterisk to it, saves in new location.
pub fn deasterisk_tavern_file(png_path: &Path, auto_overwrite: bool) -> Result<()> {
    println!("Deasterisk file: {}", &png_path.display());
    let image_data = read_image_from_file(png_path)?;
    let mut card = TavernCardV2::from_png_image(&image_data)?;
    println!(
        "Character name is {}",
        card.data.name.to_owned().unwrap_or_else(|| "".to_string())
    );
    deasterisk_tavern_card(&mut card);

    info!("\nCHARACTER INFO:\n{:#?}", &card.data);

    // Build new file name.
    let file_name = png_path
        .file_name()
        .map_or("image.png".to_string(), |x| x.to_string_lossy().to_string());
    let new_file_name = format!("de8.{}", file_name);
    println!("Output file name: {}", new_file_name);
    let new_path = png_path.with_file_name(new_file_name);

    // Check if output file already exists
    let path_exists = new_path.try_exists();
    if let Err(e) = path_exists {
        bail!("Output path is not available: {}", e);
    }
    // If it exists - ask if it should be overwritten, unless `auto_overwrite`
    // is set to true (then always overwrite).
    if path_exists.unwrap() {
        let mut overwrite = auto_overwrite;
        if !auto_overwrite {
            println!("File {} already exists. Overwrite?", new_path.display());
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            overwrite = input.trim().to_lowercase() == "y";
        }
        if overwrite {
            std::fs::remove_file(&new_path)?;
        } else {
            bail!(
                "File {} already exists ",    new_path.display()
            );
        }
    };

    // Save image to new name
    let new_image = card.into_png_image()?;
    tools::write_image_to_file(&new_image, &new_path)?;
    println!("Done");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_paired_asterisks() {
        assert_eq!(
            remove_paired_asterisks("Hello *world*, this is a **test** of *asterisks*."),
            "Hello world, this is a **test** of asterisks."
        );
        assert_eq!(
            remove_paired_asterisks("*This* is **bold** and *this* is *italic*."),
            "This is **bold** and this is italic."
        );
        assert_eq!(
            remove_paired_asterisks("No asterisks here."),
            "No asterisks here."
        );
        assert_eq!(
            remove_paired_asterisks("Only *paired* asterisks."),
            "Only paired asterisks."
        );
        assert_eq!(
            remove_paired_asterisks("Only **unpaired** asterisks."),
            "Only **unpaired** asterisks."
        );
        assert_eq!(
            remove_paired_asterisks("Single* asterisk."),
            "Single* asterisk."
        );
        assert_eq!(
            remove_paired_asterisks("***Triple*** asterisks."),
            "***Triple*** asterisks."
        );
        assert_eq!(
            remove_paired_asterisks("*Example text of no importance*"),
            "Example text of no importance"
        );
        assert_eq!(
            remove_paired_asterisks("**Example text of no importance**"),
            "**Example text of no importance**"
        );
    }

    use crate::tavern_card_v2::*;

    #[test]
    fn test_deasterisk_tavern_card() {
        let mut card = TavernCardV2::new();
        card.data.description = Some(String::from(
            "Hello *world*, this is a **test** of *asterisks*.",
        ));
        card.data.personality = Some(String::from("*This* is **bold** and *this* is *italic*."));
        card.data.scenario = Some(String::from("No asterisks here."));
        card.data.first_mes = Some(String::from("Only *paired* asterisks."));
        card.data.mes_example = Some(String::from("Only **unpaired** asterisks."));
        card.data.character_book = Some(CharacterBook::default());
        //card.data.character_book.unwrap().entries
        let mut entry1 = CharacterBookEntry::default();
        entry1.content = String::from("*Example text of no importance*");

        let mut entry2 = CharacterBookEntry::default();
        entry2.content = String::from("**Example text of no importance**");

        card.data
            .character_book
            .as_mut()
            .unwrap()
            .entries
            .push(entry1);
        card.data
            .character_book
            .as_mut()
            .unwrap()
            .entries
            .push(entry2);

        deasterisk_tavern_card(&mut card);

        assert_eq!(
            card.data.description,
            Some(String::from(
                "Hello world, this is a **test** of asterisks."
            ))
        );
        assert_eq!(
            card.data.personality,
            Some(String::from("This is **bold** and this is italic."))
        );
        assert_eq!(card.data.scenario, Some(String::from("No asterisks here.")));
        assert_eq!(
            card.data.first_mes,
            Some(String::from("Only paired asterisks."))
        );
        assert_eq!(
            card.data.mes_example,
            Some(String::from("Only **unpaired** asterisks."))
        );
        assert_eq!(
            card.data.character_book.as_ref().unwrap().entries[0].content,
            String::from("Example text of no importance")
        );
        assert_eq!(
            card.data.character_book.as_ref().unwrap().entries[1].content,
            String::from("**Example text of no importance**")
        );
    }
}
