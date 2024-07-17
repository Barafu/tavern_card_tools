//! Functions that will likely be useful for multiple tasks
use anyhow::{bail, Context, Result};
use bytes::Bytes;
use png::text_metadata::TEXtChunk;
use std::path::Path;

/// Download web page by URL, return contents
pub fn download_page(url: &str) -> Result<String> {
    let response = reqwest::blocking::get(url)?;
    if response.status().is_success() {
        let body = response.text()?;
        return Ok(body);
    } else {
        bail!("Failed to download the web page: {:?}", response.status());
    }
}

/// Download image from URL.
pub fn download_image(url: &str) -> Result<Bytes> {
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
    Ok(downloaded_image)
}

pub fn write_image_to_file(image_data: &Bytes, image_path: &Path) -> Result<()> {
    let mut file = std::fs::File::create(image_path)?;
    std::io::Write::write_all(&mut file, &image_data)?;
    Ok(())
}

pub fn read_image_from_file(image_path: &Path) -> Result<Bytes> {
    let image_data = std::fs::read(image_path)?;
    Ok(Bytes::from(image_data))
}


/// Convert an image to PNG format.
///
/// Take an image in any supported format and convert it to PNG.
/// If the input image is already in PNG format, return the original data.
pub fn convert_to_png(image_data: &Bytes) -> Result<Bytes> {
    // Try to determine the format of the input image
    let format = image::guess_format(image_data.as_ref())?;
    // If it's already a PNG, return the original data
    if format == image::ImageFormat::Png {
        return Ok(image_data.clone());
    }
    // If it's not a PNG, decode the image
    let img = image::load_from_memory(image_data.as_ref())?;
    // Prepare a buffer to store the PNG output
    let mut png_buffer = Vec::new();
    // Convert the image to PNG and write it to the buffer
    img.write_to(
        &mut std::io::Cursor::new(&mut png_buffer),
        image::ImageFormat::Png,
    )?;
    let png_output: Bytes = Bytes::from(png_buffer);
    Ok(png_output)
}

/// Return the default image (in PNG format)
pub fn get_default_image() -> Bytes {
    Bytes::from_static(include_bytes!("no_face.png"))
}

/// Adds a key-value tEXt chunk to PNG.
///
/// Returns error if the data is not a proper PNG. Makes sure not to duplicate 
/// the text chunk with the same key.
pub fn write_text_to_png(key: &str, value: &str, image_data: &Bytes) -> Result<Bytes> {
    // # Decode
    // The decoder is a build for reader and can be used to set various decoding options
    // via `Transformations`. The default output transformation is `Transformations::IDENTITY`.
    let decoder = png::Decoder::new(image_data.as_ref());
    let mut reader = decoder.read_info()?;
    let png_info = reader.info().clone();
    
    // # Encode
    // let path_out = image_path.with_file_name("output2");
    let mut output_vec: Vec<u8> = Vec::new();
    
    // Get defaults for interlaced parameter.
    let mut info_out = png_info.clone();
    let info_default = png::Info::default();
    info_out.interlaced = info_default.interlaced;

    // Add text entry. Make sure only one text entry with that key exists.     
    info_out.uncompressed_latin1_text.retain(|x|x.keyword != key);
    let new_text_entry = TEXtChunk { keyword: key.to_string(), text: value.to_string() };
    info_out.uncompressed_latin1_text.push(new_text_entry);
    
    let mut encoder = png::Encoder::with_info(&mut output_vec, info_out)?;
    encoder.set_depth(png_info.bit_depth);
    
    // Save picture with changed info. Copy frames from reader.
    let mut writer = encoder.write_header()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    while let Ok(info) = reader.next_frame(&mut buf) {
        let frame_bytes = &buf[..info.buffer_size()];
        writer.write_image_data(&frame_bytes)?;
    }
    drop(buf);
    drop(writer);
    Ok(Bytes::from(output_vec))
}

/// Searches PNG image for a tEXt chunk with a given key
pub fn read_text_chunk(image_data: &Bytes, chunk_key: &str) -> Result<Option<String>> {
    // Create a decoder
    let decoder = png::Decoder::new(image_data.as_ref());
    let reader = decoder.read_info()?;
    let png_info = reader.info();

    for text_chunk in &png_info.uncompressed_latin1_text {
        if text_chunk.keyword == chunk_key {
            return Ok(Some(text_chunk.text.clone()));
        }
    }
    // If we didn't find the chunk, return None
    Ok(None)
}