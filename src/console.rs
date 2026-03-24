use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;

const HELP_TEXT: &str = r#"
========================================
 Discord Bot - コンソールコマンド一覧
========================================
 /exclusionserver        サーバーを除外リストに追加
 /removeserver           サーバーを除外リストから削除
 /exclusionchannel       チャンネルを除外リストに追加
 /removechannel          チャンネルを除外リストから削除
 /listexclusions         現在の除外リストを表示
 /help                   このヘルプを表示
 /quit                   Botを終了
========================================
"#;

/// コンソール入力ループを非同期で起動する
pub async fn run_console(config: Arc<RwLock<Config>>) {
    println!("コンソール起動。'/help' でコマンド一覧を表示します。");

    loop {
        // プロンプト表示
        print!("> ");
        io::stdout().flush().unwrap();

        // stdin からの読み取りは blocking なので spawn_blocking を使う
        let line = tokio::task::spawn_blocking(|| {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).ok();
            buf.trim().to_string()
        })
        .await
        .unwrap_or_default();

        match line.as_str() {
            "/help" => {
                println!("{}", HELP_TEXT);
            }

            "/exclusionserver" => {
                let id = prompt_blocking("除外するサーバーIDを入力してください: ").await;
                if id.is_empty() {
                    println!("キャンセルしました。");
                    continue;
                }
                let mut cfg: tokio::sync::RwLockWriteGuard<Config> = config.write().await;
                if cfg.add_excluded_server(&id) {
                    if let Err(e) = cfg.save() {
                        eprintln!("config.json の保存に失敗しました: {}", e);
                    } else {
                        println!("サーバー {} を除外リストに追加しました。", id);
                    }
                } else {
                    println!("サーバー {} はすでに除外リストに含まれています。", id);
                }
            }

            "/removeserver" => {
                let id = prompt_blocking("除外リストから削除するサーバーIDを入力してください: ").await;
                if id.is_empty() {
                    println!("キャンセルしました。");
                    continue;
                }
                let mut cfg: tokio::sync::RwLockWriteGuard<Config> = config.write().await;
                if cfg.remove_excluded_server(&id) {
                    if let Err(e) = cfg.save() {
                        eprintln!("config.json の保存に失敗しました: {}", e);
                    } else {
                        println!("サーバー {} を除外リストから削除しました。", id);
                    }
                } else {
                    println!("サーバー {} は除外リストに存在しません。", id);
                }
            }

            "/exclusionchannel" => {
                let id = prompt_blocking("除外するチャンネルIDを入力してください: ").await;
                if id.is_empty() {
                    println!("キャンセルしました。");
                    continue;
                }
                let mut cfg: tokio::sync::RwLockWriteGuard<Config> = config.write().await;
                if cfg.add_excluded_channel(&id) {
                    if let Err(e) = cfg.save() {
                        eprintln!("config.json の保存に失敗しました: {}", e);
                    } else {
                        println!("チャンネル {} を除外リストに追加しました。", id);
                    }
                } else {
                    println!("チャンネル {} はすでに除外リストに含まれています。", id);
                }
            }

            "/removechannel" => {
                let id = prompt_blocking("除外リストから削除するチャンネルIDを入力してください: ").await;
                if id.is_empty() {
                    println!("キャンセルしました。");
                    continue;
                }
                let mut cfg: tokio::sync::RwLockWriteGuard<Config> = config.write().await;
                if cfg.remove_excluded_channel(&id) {
                    if let Err(e) = cfg.save() {
                        eprintln!("config.json の保存に失敗しました: {}", e);
                    } else {
                        println!("チャンネル {} を除外リストから削除しました。", id);
                    }
                } else {
                    println!("チャンネル {} は除外リストに存在しません。", id);
                }
            }

            "/listexclusions" => {
                let cfg = config.read().await;
                println!("\n--- 除外サーバー一覧 ---");
                if cfg.server.exclusion.is_empty() {
                    println!("  (なし)");
                } else {
                    for id in &cfg.server.exclusion {
                        println!("  - {}", id);
                    }
                }
                println!("--- 除外チャンネル一覧 ---");
                if cfg.server.channel.exclusion.is_empty() {
                    println!("  (なし)");
                } else {
                    for id in &cfg.server.channel.exclusion {
                        println!("  - {}", id);
                    }
                }
                println!();
            }

            "/quit" | "/exit" => {
                println!("Botを終了します。");
                std::process::exit(0);
            }

            "" => {}

            other => {
                println!("不明なコマンド: `{}` '/help' でコマンド一覧を確認できます。", other);
            }
        }
    }
}

/// 非同期でプロンプトを表示して1行読み取る
async fn prompt_blocking(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();

    tokio::task::spawn_blocking(|| {
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).ok();
        buf.trim().to_string()
    })
    .await
    .unwrap_or_default()
}