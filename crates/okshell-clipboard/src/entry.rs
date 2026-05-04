use std::hash::{Hash, Hasher};

use time::OffsetDateTime;

#[derive(Clone, Debug)]
pub struct ClipboardEntry {
    pub id: u64,
    pub timestamp: OffsetDateTime,
    pub mime_type: String,
    pub data: Vec<u8>,
    pub preview: EntryPreview,
    pub content_hash: u64,
}

impl ClipboardEntry {
    pub fn content_hash(data: &[u8]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Clone, Debug)]
pub enum EntryPreview {
    Text(String),
    Image {
        rgba: Vec<u8>,
        width: u32,
        height: u32,
    },
    Binary {
        mime_type: String,
        size: usize,
    },
}

impl EntryPreview {
    pub const TEXT_PREVIEW_LEN: usize = 200;
    pub const THUMBNAIL_SIZE: u32 = 512;
}
