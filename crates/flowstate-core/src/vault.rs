use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::CoreResult;

/// Local credential file (0600). Not stored in SQLite or protocol events.
pub struct CredentialVault {
    path: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CredentialStore {
    providers: HashMap<String, String>,
}

impl CredentialVault {
    pub fn new(path: PathBuf) -> CoreResult<Self> {
        Ok(Self { path })
    }

    pub fn set_api_key(&self, provider_id: &str, api_key: &str) -> CoreResult<()> {
        let mut store = self.load()?;
        store
            .providers
            .insert(provider_id.to_string(), api_key.to_string());
        self.save(&store)
    }

    pub fn get_api_key(&self, provider_id: &str) -> CoreResult<Option<String>> {
        let store = self.load()?;
        Ok(store.providers.get(provider_id).cloned())
    }

    pub fn remove_api_key(&self, provider_id: &str) -> CoreResult<()> {
        let mut store = self.load()?;
        store.providers.remove(provider_id);
        self.save(&store)
    }

    pub fn has_credential(&self, provider_id: &str) -> CoreResult<bool> {
        Ok(self.get_api_key(provider_id)?.is_some())
    }

    fn load(&self) -> CoreResult<CredentialStore> {
        if !self.path.exists() {
            return Ok(CredentialStore::default());
        }
        let raw = fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    fn save(&self, store: &CredentialStore) -> CoreResult<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string(store)?;
        fs::write(&self.path, raw)?;
        restrict_permissions(&self.path)?;
        Ok(())
    }
}

#[cfg(unix)]
fn restrict_permissions(path: &PathBuf) -> CoreResult<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &PathBuf) -> CoreResult<()> {
    Ok(())
}
