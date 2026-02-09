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
    // 默认玩家档案
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
    // 默认永久升级数据
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
    // 创建存档管理器（默认使用 save 目录）
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("save"),
        }
    }

    // 确保存档目录存在
    pub fn ensure_dirs(&self) -> io::Result<()> {
        fs::create_dir_all(&self.root)
    }

    // 玩家档案路径
    pub fn profile_path(&self) -> PathBuf {
        self.root.join("profile.json")
    }

    // 排行榜路径
    pub fn leaderboard_path(&self) -> PathBuf {
        self.root.join("leaderboard.json")
    }

    // 读取玩家档案
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

    // 保存玩家档案
    pub fn save_profile(&self, profile: &PlayerProfile) -> io::Result<()> {
        self.ensure_dirs()?;
        let path = self.profile_path();
        write_json_atomic(&path, profile)
    }

    // 读取排行榜
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

    // 保存排行榜
    pub fn save_leaderboard(&self, leaderboard: &Leaderboard) -> io::Result<()> {
        self.ensure_dirs()?;
        let path = self.leaderboard_path();
        write_json_atomic(&path, leaderboard)
    }
}

// 原子写入JSON：先写临时文件，再替换正式文件
fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    // 序列化为可读的JSON
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    // 临时文件写入成功后再替换，避免损坏
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(tmp, path)?;
    Ok(())
}
