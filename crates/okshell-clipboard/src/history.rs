use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::entry::ClipboardEntry;

#[derive(Clone)]
pub struct ClipboardHistory {
    inner: Arc<Mutex<HistoryInner>>,
}

struct HistoryInner {
    entries: VecDeque<ClipboardEntry>,
    max_entries: usize,
    next_id: u64,
}

impl ClipboardHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HistoryInner {
                entries: VecDeque::with_capacity(max_entries),
                max_entries,
                next_id: 1,
            })),
        }
    }

    /// Push a new entry, deduplicating against existing entries.
    ///
    /// If an entry with the same content hash already exists, it is moved
    /// to the front (most recent) and its timestamp is updated, rather than
    /// inserting a duplicate.
    ///
    /// Returns the id of the (new or promoted) entry.
    pub fn push(&self, mut entry: ClipboardEntry) -> u64 {
        let mut inner = self.inner.lock().unwrap();

        // Deduplicate: if the same content exists, remove it so we can
        // re-insert at the front with an updated timestamp.
        if let Some(pos) = inner
            .entries
            .iter()
            .position(|e| e.content_hash == entry.content_hash)
        {
            let existing = inner.entries.remove(pos).unwrap();
            // Reuse the existing id so any external references stay valid.
            entry.id = existing.id;
            inner.entries.push_front(entry);
            return inner.entries.front().unwrap().id;
        }

        entry.id = inner.next_id;
        inner.next_id += 1;

        // Remove oldest if at capacity.
        if inner.entries.len() >= inner.max_entries {
            inner.entries.pop_back();
        }

        let id = entry.id;
        inner.entries.push_front(entry);
        id
    }

    pub fn entries(&self) -> Vec<ClipboardEntry> {
        let inner = self.inner.lock().unwrap();
        inner.entries.iter().cloned().collect()
    }

    pub fn get(&self, id: u64) -> Option<ClipboardEntry> {
        let inner = self.inner.lock().unwrap();
        inner.entries.iter().find(|e| e.id == id).cloned()
    }

    pub fn remove(&self, id: u64) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if let Some(pos) = inner.entries.iter().position(|e| e.id == id) {
            inner.entries.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.entries.clear();
    }

    /// Move an entry to the front of the history without changing its id.
    pub fn promote(&self, id: u64) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(pos) = inner.entries.iter().position(|e| e.id == id) {
            let mut entry = inner.entries.remove(pos).unwrap();
            entry.timestamp = time::OffsetDateTime::now_utc();
            inner.entries.push_front(entry);
        }
    }
}
