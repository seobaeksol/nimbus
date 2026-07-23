use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use crate::model::AppState;

#[derive(Clone, Debug)]
pub struct StateStore {
    path: PathBuf,
}

impl StateStore {
    pub fn for_current_user() -> Self {
        let base = env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| env::var_os("LOCALAPPDATA").map(PathBuf::from))
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            path: base.join("Nimbus").join("state.json"),
        }
    }

    #[cfg(test)]
    fn from_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(
        &self,
        initial_path: PathBuf,
        requested_workspace: Option<&str>,
    ) -> (AppState, Option<String>) {
        let mut state = match fs::read(&self.path) {
            Ok(bytes) => match serde_json::from_slice::<AppState>(&bytes) {
                Ok(state) => state,
                Err(error) => {
                    let mut state = AppState::new(initial_path.clone());
                    state.normalize(&initial_path);
                    return (
                        state,
                        Some(format!(
                            "Nimbus could not restore its saved workspace: {error}. A fresh workspace was opened."
                        )),
                    );
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                AppState::new(initial_path.clone())
            }
            Err(error) => {
                let mut state = AppState::new(initial_path.clone());
                state.normalize(&initial_path);
                return (
                    state,
                    Some(format!(
                        "Nimbus could not read {}: {error}",
                        self.path.display()
                    )),
                );
            }
        };

        state.normalize(&initial_path);
        let warning = requested_workspace.and_then(|name| {
            if state.select_workspace(name) {
                None
            } else {
                Some(format!(
                    "Workspace “{name}” was not found. Restored “{}” instead.",
                    state.workspace().name
                ))
            }
        });
        (state, warning)
    }

    pub fn save(&self, state: &AppState) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut persisted = state.clone();
        for shelf in &mut persisted.shelves {
            if !shelf.persistent {
                shelf.items.clear();
            }
        }

        let bytes = serde_json::to_vec_pretty(&persisted)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let temporary = self.path.with_extension("json.tmp");
        let backup = self.path.with_extension("json.bak");
        fs::write(&temporary, bytes)?;

        if self.path.exists() {
            let _ = fs::copy(&self.path, &backup);
            match fs::rename(&temporary, &self.path) {
                Ok(()) => return Ok(()),
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::AlreadyExists | io::ErrorKind::PermissionDenied
                    ) => {}
                Err(error) => return Err(error),
            }
            fs::remove_file(&self.path)?;
        }
        fs::rename(temporary, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::model::{LayoutPreset, ShelfItem};

    fn temp_state_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir()
            .join(format!("nimbus-test-{name}-{nonce}"))
            .join("state.json")
    }

    #[test]
    fn round_trip_preserves_workspace_layout() {
        let path = temp_state_path("round-trip");
        let store = StateStore::from_path(path.clone());
        let mut state = AppState::new(PathBuf::from("C:\\work"));
        state.workspace_mut().apply_preset(LayoutPreset::Grid);
        store.save(&state).unwrap();

        let (loaded, warning) = store.load(PathBuf::from("C:\\"), None);
        assert!(warning.is_none());
        assert_eq!(loaded.workspace().panels.len(), 4);

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn session_shelf_items_are_not_persisted() {
        let path = temp_state_path("session-shelf");
        let store = StateStore::from_path(path.clone());
        let mut state = AppState::new(PathBuf::from("C:\\work"));
        state.shelf_mut().persistent = false;
        state.shelf_mut().items.push(ShelfItem {
            path: PathBuf::from("C:\\work\\draft.txt"),
            note: String::new(),
            color: None,
        });
        store.save(&state).unwrap();

        let (loaded, _) = store.load(PathBuf::from("C:\\"), None);
        assert!(loaded.shelf().items.is_empty());

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
