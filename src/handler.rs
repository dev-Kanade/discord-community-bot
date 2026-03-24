use serenity::async_trait;
use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;

pub struct BotHandler {
    pub config: Arc<RwLock<Config>>,
    pub forbidden_words: Arc<RwLock<Vec<String>>>,
}

#[async_trait]
impl EventHandler for BotHandler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        // serenity 0.12 で discriminator は Option<NonZero<u16>>
        let tag = match ready.user.discriminator {
            Some(d) => format!("{}#{:04}", ready.user.name, d),
            None => ready.user.name.clone(), // 新形式ユーザー名（discriminator なし）
        };
        println!("[Bot] {} としてログインしました", tag);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Bot 自身のメッセージは無視
        if msg.author.bot {
            return;
        }

        let config = self.config.read().await;

        // 禁止ワード機能が無効なら何もしない
        if !config.forbidden_words_enabled {
            return;
        }

        // サーバー除外チェック
        if let Some(guild_id) = msg.guild_id {
            if config.server.exclusion.contains(&guild_id.to_string()) {
                return;
            }
        }

        // チャンネル除外チェック
        if config
            .server
            .channel
            .exclusion
            .contains(&msg.channel_id.to_string())
        {
            return;
        }

        // 禁止ワードチェック
        let words = self.forbidden_words.read().await;
        let content_lower = msg.content.to_lowercase();
        let matched = words
            .iter()
            .any(|w| content_lower.contains(&w.to_lowercase()));

        if !matched {
            return;
        }
        drop(words);

        // サーバー名を取得
        let guild_name = if let Some(guild_id) = msg.guild_id {
            ctx.cache
                .guild(guild_id)
                .map(|g| g.name.clone())
                .unwrap_or_else(|| "不明なサーバー".to_string())
        } else {
            "不明なサーバー".to_string()
        };

        drop(config);

        // メッセージを削除
        if let Err(e) = msg.delete(&ctx.http).await {
            eprintln!("[Bot] メッセージ削除失敗: {}", e);
            return;
        }

        // DM チャンネルを開く
        let dm_channel = match msg.author.create_dm_channel(&ctx.http).await {
            Ok(ch) => ch,
            Err(e) => {
                eprintln!(
                    "[Bot] DMチャンネル作成失敗 (ユーザー: {}): {}",
                    msg.author.name, e
                );
                return;
            }
        };

        // serenity 0.12: CreateEmbed / CreateMessage を直接構築（クロージャ廃止）
        let embed = CreateEmbed::new()
            .title("措置が取られました！")
            .description(format!(
                "あなたが`{}`で送信した文章には禁止されているワードが含まれていたため該当のメッセージを削除しました。",
                guild_name
            ))
            .color(0xFF0000u32); // 赤

        let builder = CreateMessage::new().embed(embed);

        if let Err(e) = dm_channel.send_message(&ctx.http, builder).await {
            eprintln!(
                "[Bot] DM送信失敗 (ユーザー: {}): {}",
                msg.author.name, e
            );
        }
    }
}