use std::{collections::HashMap, ffi::OsStr, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Website root; used when generating links.
    /// Can also be specified using the environment variable `$TREEHOUSE_SITE`. (this is the
    /// preferred way of setting this in production, so as not to clobber treehouse.toml.)
    pub site: String,

    /// User-defined keys.
    pub user: HashMap<String, String>,

    /// Links exported to Markdown for use with reference syntax `[text][def:key]`.
    pub defs: HashMap<String, String>,

    /// Overrides for emoji filenames. Useful for setting up aliases.
    ///
    /// On top of this, emojis are autodiscovered by walking the `static/emoji` directory.
    #[serde(default)]
    pub emoji: HashMap<String, String>,

    /// Overrides for pic filenames. Useful for setting up aliases.
    ///
    /// On top of this, pics are autodiscovered by walking the `static/pic` directory.
    /// Only the part before the first dash is treated as the pic's id.
    pub pics: HashMap<String, String>,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let string = std::fs::read_to_string(path).context("cannot read config file")?;
        toml_edit::de::from_str(&string).context("error in config file")
    }

    fn is_emoji_file(path: &Path) -> bool {
        path.extension() == Some(OsStr::new("png")) || path.extension() == Some(OsStr::new("svg"))
    }

    pub fn autopopulate_emoji(&mut self, dir: &Path) -> anyhow::Result<()> {
        for file in WalkDir::new(dir) {
            let entry = file?;
            if entry.file_type().is_file() && Self::is_emoji_file(entry.path()) {
                if let Some(emoji_name) = entry.path().file_stem() {
                    let emoji_name = emoji_name.to_string_lossy();
                    if !self.emoji.contains_key(emoji_name.as_ref()) {
                        self.emoji.insert(
                            emoji_name.into_owned(),
                            entry
                                .path()
                                .strip_prefix(dir)
                                .unwrap_or(entry.path())
                                .to_string_lossy()
                                .into_owned(),
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn is_pic_file(path: &Path) -> bool {
        path.extension() == Some(OsStr::new("png"))
            || path.extension() == Some(OsStr::new("svg"))
            || path.extension() == Some(OsStr::new("jpg"))
            || path.extension() == Some(OsStr::new("jpeg"))
            || path.extension() == Some(OsStr::new("webp"))
    }

    pub fn autopopulate_pics(&mut self, dir: &Path) -> anyhow::Result<()> {
        for file in WalkDir::new(dir) {
            let entry = file?;
            if entry.file_type().is_file() && Self::is_pic_file(entry.path()) {
                if let Some(pic_name) = entry.path().file_stem() {
                    let pic_name = pic_name.to_string_lossy();

                    let pic_id = pic_name
                        .split_once('-')
                        .map(|(before_dash, _after_dash)| before_dash)
                        .unwrap_or(&pic_name);

                    if !self.pics.contains_key(pic_id) {
                        self.pics.insert(
                            pic_id.to_owned(),
                            entry
                                .path()
                                .strip_prefix(dir)
                                .unwrap_or(entry.path())
                                .to_string_lossy()
                                .into_owned(),
                        );
                    }
                }
            }
        }
        Ok(())
    }
}
