//! Functions that will likely be useful for multiple tasks
use anyhow::{bail, Context, Result};
use bytes::Bytes;
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

/// Add a key-value tEXt chunk to PNG.
///
/// Return error if the data is not a proper PNG.
pub fn write_text_to_png(key: &str, value: &str, image_data: &Bytes) -> Result<Bytes> {
    // # Decode
    // The decoder is a build for reader and can be used to set various decoding options
    // via `Transformations`. The default output transformation is `Transformations::IDENTITY`.
    let decoder = png::Decoder::new(image_data.as_ref());
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
    let mut output_vec: Vec<u8> = Vec::new();

    // Get defaults for interlaced parameter.
    let mut info_out = png_info.clone();
    let info_default = png::Info::default();

    // Edit previous info
    info_out.interlaced = info_default.interlaced;
    let mut encoder = png::Encoder::with_info(&mut output_vec, info_out)?;
    encoder.set_depth(png_info.bit_depth);

    // Edit test attribute
    encoder.add_text_chunk(key.to_string(), value.to_string())?;

    // Save picture with changed info
    let mut writer = encoder.write_header()?;
    for frame in memory {
        writer.write_image_data(&frame)?;
    }
    drop(writer);
    Ok(Bytes::from(output_vec))
}
