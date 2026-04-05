use image::ImageReader;
use std::io::Cursor;

use crate::entry::EntryPreview;

pub fn generate_thumbnail(data: &[u8]) -> Option<EntryPreview> {
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()?;

    let img = reader.decode().ok()?;

    let thumb = img.thumbnail(
        EntryPreview::THUMBNAIL_SIZE,
        EntryPreview::THUMBNAIL_SIZE,
    );

    let rgba = thumb.to_rgba8();
    let (width, height) = rgba.dimensions();

    Some(EntryPreview::Image {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}