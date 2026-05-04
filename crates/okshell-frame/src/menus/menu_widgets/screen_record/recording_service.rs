use okshell_screenshot::record::RecordHandle;
use reactive_stores::{ArcStore, KeyMap, PatchField, Store, StorePath};
use std::sync::OnceLock;

#[derive(Clone, Default, Store)]
pub struct RecordingState {
    pub handle: Option<RecordHandle>,
}

impl PatchField for RecordingState {
    fn patch_field(
        &mut self,
        new: Self,
        path: &StorePath,
        notify: &mut dyn FnMut(&StorePath),
        _keys: Option<&KeyMap>,
    ) {
        // No way to compare RecordHandles, so always replace and notify
        self.handle = new.handle;
        notify(path);
    }
}

static RECORDING_STATE: OnceLock<ArcStore<RecordingState>> = OnceLock::new();

pub fn recording_state() -> &'static ArcStore<RecordingState> {
    RECORDING_STATE.get_or_init(|| ArcStore::new(RecordingState::default()))
}
