use serde::{Deserialize, Serialize};
use std::fs;

const CONFIG_PATH: &str = "config.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub forbidden_words_enabled: bool,
    pub server: ServerConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub exclusion: Vec<String>,
    pub channel: ChannelConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelConfig {
    pub exclusion: Vec<String>,
}

impl Config {
    /// config.json を読み込む
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let text = fs::read_to_string(CONFIG_PATH)?;
        let config = serde_json::from_str(&text)?;
        Ok(config)
    }

    /// config.json に保存する
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(CONFIG_PATH, json)?;
        Ok(())
    }

    /// サーバーを除外リストに追加（重複無視）
    pub fn add_excluded_server(&mut self, server_id: &str) -> bool {
        if self.server.exclusion.contains(&server_id.to_string()) {
            return false; // 既に登録済み
        }
        self.server.exclusion.push(server_id.to_string());
        true
    }

    /// チャンネルを除外リストに追加（重複無視）
    pub fn add_excluded_channel(&mut self, channel_id: &str) -> bool {
        if self.server.channel.exclusion.contains(&channel_id.to_string()) {
            return false;
        }
        self.server.channel.exclusion.push(channel_id.to_string());
        true
    }

    /// サーバー除外リストから削除
    pub fn remove_excluded_server(&mut self, server_id: &str) -> bool {
        let before = self.server.exclusion.len();
        self.server.exclusion.retain(|id| id != server_id);
        self.server.exclusion.len() < before
    }

    /// チャンネル除外リストから削除
    pub fn remove_excluded_channel(&mut self, channel_id: &str) -> bool {
        let before = self.server.channel.exclusion.len();
        self.server.channel.exclusion.retain(|id| id != channel_id);
        self.server.channel.exclusion.len() < before
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            forbidden_words_enabled: false,
            server: ServerConfig {
                exclusion: vec![],
                channel: ChannelConfig { exclusion: vec![] },
            },
        }
    }
}