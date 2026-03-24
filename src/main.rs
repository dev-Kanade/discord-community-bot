mod config;
mod console;
mod handler;
mod setup;

use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use serenity::prelude::*;

use config::Config;
use handler::BotHandler;

#[tokio::main]
async fn main() {
    // ── 初回セットアップ ──────────────────────────────────────────
    let is_first_run = !Path::new(".env").exists() || !Path::new("config.json").exists();

    if is_first_run {
        println!("=== 初回セットアップ ===");
        if let Err(e) = setup::run_setup().await {
            eprintln!("セットアップ中にエラーが発生しました: {}", e);
            std::process::exit(1);
        }
        println!("\nセットアップが完了しました。Botを起動します...\n");
    }

    // ── 環境変数と設定ファイルの読み込み ──────────────────────────
    dotenvy::dotenv().ok();

    let token = std::env::var("DISCORDBOTTOKEN").unwrap_or_else(|_| {
        eprintln!("エラー: DISCORDBOTTOKEN が .env に見つかりません。");
        std::process::exit(1);
    });

    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("config.json の読み込みに失敗しました: {}", e);
        std::process::exit(1);
    });

    // ── 禁止ワードリストの読み込み ────────────────────────────────
    let forbidden_words: Vec<String> = if Path::new("forbidden_words.json").exists() {
        let text = std::fs::read_to_string("forbidden_words.json").unwrap_or_default();
        serde_json::from_str(&text).unwrap_or_default()
    } else {
        vec![]
    };

    println!("[Bot] 禁止ワード: {} 件読み込み", forbidden_words.len());

    // ── 共有状態を Arc<RwLock<T>> でラップ ───────────────────────
    let config = Arc::new(RwLock::new(config));
    let forbidden_words = Arc::new(RwLock::new(forbidden_words));

    // ── コンソールループを別タスクで起動 ─────────────────────────
    let config_for_console = Arc::clone(&config);
    tokio::spawn(async move {
        console::run_console(config_for_console).await;
    });

    // ── Discord Bot クライアントの構築と起動 ──────────────────────
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let handler = BotHandler {
        config: Arc::clone(&config),
        forbidden_words: Arc::clone(&forbidden_words),
    };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Botクライアントの作成に失敗しました: {}", e);
            std::process::exit(1);
        });

    println!("[Bot] 起動中...");

    if let Err(e) = client.start().await {
        eprintln!("[Bot] 実行中にエラーが発生しました: {}", e);
    }
}