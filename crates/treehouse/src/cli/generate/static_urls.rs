use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
    path::PathBuf,
    sync::RwLock,
};

use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use serde_json::Value;

pub struct StaticUrls {
    base_dir: PathBuf,
    base_url: String,
    // Really annoying that we have to use an RwLock for this. We only ever generate in a
    // single-threaded environment.
    // Honestly it would be a lot more efficient if Handlebars just assumed single-threadedness
    // and required you to clone it over to different threads.
    // Stuff like this is why I really want to implement my own templating engine...
    hash_cache: RwLock<HashMap<String, String>>,
}

impl StaticUrls {
    pub fn new(base_dir: PathBuf, base_url: String) -> Self {
        Self {
            base_dir,
            base_url,
            hash_cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, filename: &str) -> Result<String, io::Error> {
        let hash_cache = self.hash_cache.read().unwrap();
        if let Some(cached) = hash_cache.get(filename) {
            return Ok(cached.to_owned());
        }
        drop(hash_cache);

        let mut hasher = blake3::Hasher::new();
        let file = BufReader::new(File::open(self.base_dir.join(filename))?);
        hasher.update_reader(file)?;
        // NOTE: Here the hash is truncated to 8 characters. This is fine, because we don't
        // care about security here - only detecting changes in files.
        let hash = format!(
            "{}/{}?cache=b3-{}",
            self.base_url,
            filename,
            &hasher.finalize().to_hex()[0..8]
        );
        {
            let mut hash_cache = self.hash_cache.write().unwrap();
            hash_cache.insert(filename.to_owned(), hash.clone());
        }
        Ok(hash)
    }
}

impl HelperDef for StaticUrls {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        helper: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        if let Some(param) = helper.param(0).and_then(|v| v.value().as_str()) {
            return Ok(ScopedJson::Derived(Value::String(
                self.get(param).map_err(|error| {
                    RenderError::new(format!("cannot get asset url for {param}: {error}"))
                })?,
            )));
        }

        Err(RenderError::new("asset path must be provided"))
    }
}
