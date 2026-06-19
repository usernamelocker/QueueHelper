use std::{
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

    pub async fn load_or_default_with_expected_version(&self, expected_version: u32) -> Result<T> {
        if tokio::fs::metadata(&self.path).await.is_err() {
            let default_value = T::default();
            self.save(&default_value).await?;
            return Ok(default_value);
        }

        let raw = tokio::fs::read_to_string(&self.path)
            .await
            .with_context(|| format!("failed reading JSON store {}", self.path.display()))?;

        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(version) = value.get("schemaVersion").and_then(|v| v.as_u64()) {
                if version as u32 != expected_version {
                    let default_value = T::default();
                    eprintln!(
                        "[JsonStore] schema version mismatch for {}: expected {}, got {}. Resetting to defaults.",
                        self.path.display(),
                        expected_version,
                        version
                    );
                    self.save(&default_value).await?;
                    return Ok(default_value);
                }
            }
        }

        let parsed = serde_json::from_str::<T>(&raw)
            .with_context(|| format!("failed parsing JSON store {}", self.path.display()))?;
        Ok(parsed)
    }

    pub async fn load_or_default(&self) -> Result<T> {
        if tokio::fs::metadata(&self.path).await.is_err() {
            let default_value = T::default();
            self.save(&default_value).await?;
            return Ok(default_value);
        }

        let raw = tokio::fs::read_to_string(&self.path)
            .await
            .with_context(|| format!("failed reading JSON store {}", self.path.display()))?;
        let parsed = serde_json::from_str::<T>(&raw)
            .with_context(|| format!("failed parsing JSON store {}", self.path.display()))?;
        Ok(parsed)
    }

    pub async fn save(&self, value: &T) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("failed creating parent directory for {}", self.path.display())
            })?;
        }

        let serialized = serde_json::to_string_pretty(value)
            .with_context(|| format!("failed serializing JSON store {}", self.path.display()))?;
        tokio::fs::write(&self.path, serialized)
            .await
            .with_context(|| format!("failed writing JSON store {}", self.path.display()))?;
        Ok(())
    }
}

