//!  Actions that don't fit other modules.

use std::path::Path;

use anyhow::Result;

use crate::tavern_card_v2::TavernCardV2;
use crate::tools;

/// Prints the content of tavern card from a given file path
pub fn print_tavern_card_from_path(path: &Path) -> Result<()> {
    let image = tools::read_image_from_file(path)?;
    let card = TavernCardV2::from_png_image(&image)?;    
    println!("{}", card);

    Ok(())
}
