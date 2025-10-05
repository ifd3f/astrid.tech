use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DynamicSettings {
    pub current_profile_id: Uuid,
    pub profiles: HashMap<Uuid, Profile>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Profile {
    pub name: String,
    pub action: Action,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Action {
    Redirect(String),
}

impl DynamicSettings {
    pub fn current_profile(&self) -> &Profile {
        &self.profiles[&self.current_profile_id]
    }
}

impl Default for DynamicSettings {
    fn default() -> Self {
        let uuid = Uuid::new_v4();
        let mut map = HashMap::new();
        let profile = Profile {
            name: "https://astrid.tech".to_owned(),
            action: Action::Redirect("https://astrid.tech".to_owned()),
        };

        map.insert(uuid, profile);

        Self {
            current_profile_id: uuid,
            profiles: map,
        }
    }
}

#[derive(Clone)]
pub struct Persisted<T> {
    inner: Arc<RwLock<PersistedInner<T>>>,
}

pub struct PersistedInner<T> {
    path: PathBuf,
    cached: Arc<T>,
}

impl<T> Persisted<T>
where
    T: Serialize + DeserializeOwned + Default + Clone,
{
    pub async fn open(path: PathBuf) -> Self {
        let cached = match Persisted::read_file(&path).await {
            Ok(config) => config,
            Err(_) => {
                let config = T::default();
                let ser = toml::to_string_pretty(&config).expect("error while building JSON");
                tokio::fs::write(&path, ser)
                    .await
                    .expect("Failure to write");
                config
            }
        };

        Self {
            inner: Arc::new(RwLock::new(PersistedInner {
                path,
                cached: Arc::new(cached),
            })),
        }
    }

    async fn read_file(path: &Path) -> Result<T, Box<dyn Error>> {
        let file = tokio::fs::read_to_string(&path).await?;
        let json = toml::from_str(&file)?;
        Ok(json)
    }

    pub async fn store(&self, config: T) {
        let mut lock = self.inner.write().await;
        let json = toml::to_string_pretty(&config).expect("error while building JSON");
        tokio::fs::write(&lock.path, json)
            .await
            .expect("Failure to write");
        lock.cached = config.into();
    }

    pub async fn snapshot(&self) -> Arc<T> {
        self.inner.read().await.cached.clone()
    }

    pub async fn snapshot_cloned(&self) -> T {
        self.inner.read().await.cached.as_ref().clone()
    }
}
