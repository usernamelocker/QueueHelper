use std::{
    fs,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone)]
pub struct JsonStore<T> {
    path: PathBuf,
    _marker: PhantomData<T>,
}

impl<T> JsonStore<T>
where
    T: Serialize + DeserializeOwned + Default + Clone,
{
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            _marker: PhantomData,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load_or_default(&self) -> Result<T> {
        if !self.path.exists() {
            let default_value = T::default();
            self.save(&default_value)?;
            return Ok(default_value);
        }

        let raw = fs::read_to_string(&self.path)
            .with_context(|| format!("failed reading JSON store {}", self.path.display()))?;
        let parsed = serde_json::from_str::<T>(&raw)
            .with_context(|| format!("failed parsing JSON store {}", self.path.display()))?;
        Ok(parsed)
    }

    pub fn save(&self, value: &T) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed creating parent directory for {}", self.path.display())
            })?;
        }

        let serialized = serde_json::to_string_pretty(value)
            .with_context(|| format!("failed serializing JSON store {}", self.path.display()))?;
        fs::write(&self.path, serialized)
            .with_context(|| format!("failed writing JSON store {}", self.path.display()))?;
        Ok(())
    }
}

