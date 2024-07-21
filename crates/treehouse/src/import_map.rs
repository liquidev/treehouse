use std::{ffi::OsStr, path::PathBuf};

use indexmap::IndexMap;
use log::warn;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::static_urls::StaticUrls;

#[derive(Debug, Clone, Serialize)]
pub struct ImportMap {
    pub imports: IndexMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImportRoot {
    pub name: String,
    pub path: String,
}

impl ImportMap {
    pub fn generate(base_url: String, import_roots: &[ImportRoot]) -> Self {
        let mut import_map = ImportMap {
            imports: IndexMap::new(),
        };

        for root in import_roots {
            let static_urls = StaticUrls::new(
                PathBuf::from(&root.path),
                format!("{base_url}/{}", root.path),
            );
            for entry in WalkDir::new(&root.path) {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => {
                        warn!("directory walk failed: {error}");
                        continue;
                    }
                };

                if !entry.file_type().is_dir() && entry.path().extension() == Some(OsStr::new("js"))
                {
                    let normalized_path = entry
                        .path()
                        .strip_prefix(&root.path)
                        .unwrap_or(entry.path())
                        .to_string_lossy()
                        .replace('\\', "/");
                    match static_urls.get(&normalized_path) {
                        Ok(url) => {
                            import_map
                                .imports
                                .insert(format!("{}/{normalized_path}", root.name), url);
                        }
                        Err(error) => {
                            warn!("could not get static url for {normalized_path}: {error}")
                        }
                    }
                }
            }
        }

        import_map.imports.sort_unstable_keys();

        import_map
    }
}
