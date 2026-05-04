use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use relm4::gtk::glib;

static ICON_INDEX: RwLock<Option<(String, Arc<IconIndex>)>> = RwLock::new(None);

pub struct IconIndex {
    icons: HashMap<String, PathBuf>,
}

impl IconIndex {
    pub fn get_or_build(theme_name: &str) -> Arc<IconIndex> {
        {
            let guard = ICON_INDEX.read().unwrap();
            if let Some((cached_theme, index)) = guard.as_ref()
                && cached_theme == theme_name
            {
                return Arc::clone(index);
            }
        }

        let index = Arc::new(Self::build(theme_name));
        {
            let mut guard = ICON_INDEX.write().unwrap();
            *guard = Some((theme_name.to_string(), Arc::clone(&index)));
        }
        index
    }

    fn build(theme_name: &str) -> Self {
        let mut icons = HashMap::new();

        let data_dirs = glib::system_data_dirs();

        // Build search paths in priority order:
        // 1. User-local theme directory
        // 2. System theme directories
        // 3. hicolor fallback
        // 4. pixmaps fallback
        let mut search_dirs: Vec<PathBuf> = Vec::new();

        // User-local theme dir (highest priority)
        if let Some(home) = glib::home_dir().to_str().map(|s| s.to_string()) {
            let user_theme = PathBuf::from(format!("{}/.local/share/icons/{}", home, theme_name));
            if user_theme.is_dir() {
                search_dirs.push(user_theme);
            }

            let user_hicolor = PathBuf::from(format!("{}/.local/share/icons/hicolor", home));
            if user_hicolor.is_dir() {
                search_dirs.push(user_hicolor);
            }
        }

        // System theme directories
        for d in &data_dirs {
            let theme_dir = d.join("icons").join(theme_name);
            if theme_dir.is_dir() {
                search_dirs.push(theme_dir);
            }
        }

        // hicolor fallback (system)
        for d in &data_dirs {
            let hicolor = d.join("icons").join("hicolor");
            if hicolor.is_dir() {
                search_dirs.push(hicolor);
            }
        }

        // pixmaps fallback
        for d in &data_dirs {
            let pixmaps = d.join("pixmaps");
            if pixmaps.is_dir() {
                search_dirs.push(pixmaps);
            }
        }

        // Scan in reverse order so higher-priority dirs overwrite lower ones
        for base in search_dirs.iter().rev() {
            Self::scan_dir(base, &mut icons);
        }

        IconIndex { icons }
    }

    fn scan_dir(dir: &PathBuf, icons: &mut HashMap<String, PathBuf>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::scan_dir(&path, icons);
            } else {
                // Only index image files we can actually load
                let dominated_ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                match dominated_ext {
                    "png" | "svg" | "xpm" | "jpg" | "jpeg" | "webp" => {}
                    _ => continue,
                }

                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Higher-priority dirs were scanned last, so they overwrite
                    icons.insert(stem.to_string(), path);
                }
            }
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&PathBuf> {
        self.icons.get(name)
    }

    /// Invalidate the cache (e.g. when icon theme changes)
    pub fn invalidate() {
        let mut guard = ICON_INDEX.write().unwrap();
        *guard = None;
    }
}
