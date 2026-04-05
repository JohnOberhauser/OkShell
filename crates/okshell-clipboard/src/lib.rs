mod entry;
mod history;
mod thumbnail;
mod watcher;

pub use entry::{ClipboardEntry, EntryPreview};
pub use history::ClipboardHistory;
pub use watcher::{ClipboardEvent, ClipboardWatcher};

use std::sync::OnceLock;

static CLIPBOARD: OnceLock<ClipboardWatcher> = OnceLock::new();

pub fn clipboard_service() -> &'static ClipboardWatcher {
    CLIPBOARD.get_or_init(|| {
        ClipboardWatcher::start(100).expect("Failed to start clipboard watcher")
    })
}