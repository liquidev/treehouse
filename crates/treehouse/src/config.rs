use std::{collections::HashMap, ffi::OsStr, fs::File, io::BufReader, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tracing::debug;
use walkdir::WalkDir;

use crate::html::highlight::{
    compiled::{compile_syntax, CompiledSyntax},
    Syntax,
};

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

    /// Redirects for moving pages around. These are used solely by the treehouse server.
    ///
    /// Note that redirects are only resolved _non-recursively_ by the server. For a configuration
    /// like:
    ///
    /// ```toml
    /// page.redirects.foo = "bar"
    /// page.redirects.bar = "baz"
    /// ```
    ///
    /// the user will be redirected from `foo` to `bar`, then from `bar` to `baz`. This isn't
    /// optimal for UX and causes unnecessary latency. Therefore you should always make redirects
    /// point to the newest version of the page.
    ///
    /// ```toml
    /// page.redirects.foo = "baz"
    /// page.redirects.bar = "baz"
    /// ```
    pub redirects: Redirects,

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

    /// Syntax definitions.
    ///
    /// These are not part of the config file, but are loaded as part of site configuration from
    /// `static/syntax`.
    #[serde(skip)]
    pub syntaxes: HashMap<String, CompiledSyntax>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Redirects {
    /// Page redirects. When a user navigates to a page, if they navigate to `url`, they will
    /// be redirected to `page[url]`.
    pub page: HashMap<String, String>,
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

    pub fn page_url(&self, page: &str) -> String {
        format!("{}/{}", self.site, page)
    }

    pub fn pic_url(&self, id: &str) -> String {
        format!(
            "{}/static/pic/{}",
            self.site,
            self.pics.get(id).map(|x| &**x).unwrap_or("404.png")
        )
    }

    /// Loads all syntax definition files.
    pub fn load_syntaxes(&mut self, dir: &Path) -> anyhow::Result<()> {
        for entry in WalkDir::new(dir) {
            let entry = entry?;
            if entry.path().extension() == Some(OsStr::new("json")) {
                let name = entry
                    .path()
                    .file_stem()
                    .expect("syntax file name should have a stem")
                    .to_string_lossy();
                debug!("loading syntax {name:?}");

                let syntax: Syntax = serde_json::from_reader(BufReader::new(
                    File::open(entry.path()).context("could not open syntax file")?,
                ))
                .context("could not deserialize syntax file")?;
                let compiled = compile_syntax(&syntax);
                self.syntaxes.insert(name.into_owned(), compiled);
            }
        }

        Ok(())
    }
}

/// Data derived from the config.
#[derive(Debug, Clone, Default)]
pub struct ConfigDerivedData {
    pub pic_sizes: HashMap<String, Option<PicSize>>,
}

/// Picture size. This is useful for emitting <img> elements with a specific size to eliminate layout shifting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PicSize {
    pub width: u32,
    pub height: u32,
}

impl ConfigDerivedData {
    fn read_pic_size(config: &Config, pic_id: &str) -> Option<PicSize> {
        let pic_filename = config.pics.get(pic_id)?;
        let (width, height) = image::io::Reader::new(BufReader::new(
            File::open(format!("static/pic/{pic_filename}")).ok()?,
        ))
        .into_dimensions()
        .ok()?;
        Some(PicSize { width, height })
    }

    pub fn pic_size(&mut self, config: &Config, pic_id: &str) -> Option<PicSize> {
        if !self.pic_sizes.contains_key(pic_id) {
            self.pic_sizes
                .insert(pic_id.to_owned(), Self::read_pic_size(config, pic_id));
        }
        self.pic_sizes.get(pic_id).copied().flatten()
    }
}
