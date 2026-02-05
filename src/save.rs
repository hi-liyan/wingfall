use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub username: String,
    #[serde(default)]
    pub plane_style: u8,
    pub permanent: PermanentUpgrades,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            username: String::new(),
            plane_style: 0,
            permanent: PermanentUpgrades::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermanentUpgrades {
    pub bullet_level: u8,
    pub max_lives: u8,
}

impl Default for PermanentUpgrades {
    fn default() -> Self {
        Self {
            bullet_level: 1,
            max_lives: 3,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoreEntry {
    pub username: String,
    pub score: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Leaderboard {
    pub entries: Vec<ScoreEntry>,
}

pub struct SaveStore {
    root: PathBuf,
}

impl SaveStore {
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("save"),
        }
    }

    pub fn ensure_dirs(&self) -> io::Result<()> {
        fs::create_dir_all(&self.root)
    }

    pub fn profile_path(&self) -> PathBuf {
        self.root.join("profile.json")
    }

    pub fn leaderboard_path(&self) -> PathBuf {
        self.root.join("leaderboard.json")
    }

    pub fn load_profile(&self) -> io::Result<Option<PlayerProfile>> {
        let path = self.profile_path();
        if !path.exists() {
            return Ok(None);
        }
        let text = fs::read_to_string(path)?;
        let parsed: PlayerProfile = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };
        Ok(Some(parsed))
    }

    pub fn save_profile(&self, profile: &PlayerProfile) -> io::Result<()> {
        self.ensure_dirs()?;
        let path = self.profile_path();
        write_json_atomic(&path, profile)
    }

    pub fn load_leaderboard(&self) -> io::Result<Leaderboard> {
        let path = self.leaderboard_path();
        if !path.exists() {
            return Ok(Leaderboard::default());
        }
        let text = fs::read_to_string(path)?;
        let parsed: Leaderboard = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => Leaderboard::default(),
        };
        Ok(parsed)
    }

    pub fn save_leaderboard(&self, leaderboard: &Leaderboard) -> io::Result<()> {
        self.ensure_dirs()?;
        let path = self.leaderboard_path();
        write_json_atomic(&path, leaderboard)
    }
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let json = serde_json::to_string_pretty(value).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(tmp, path)?;
    Ok(())
}
