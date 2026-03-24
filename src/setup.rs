use serde::{Deserialize, Serialize};
use std::io::{self, Write};

/// config.json に保存する設定
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub forbidden_words_enabled: bool,
}

/// セットアップ処理全体を実行する
pub async fn run_setup() -> Result<(), Box<dyn std::error::Error>> {
    // --- Step 1: Discord Bot トークンの入力 ---
    let token = prompt("DiscordBotトークンを入力してください: ")?;
    write_env("DISCORDBOTTOKEN", &token)?;
    println!("トークンを .env に保存しました。");

    // --- Step 2: 禁止ワード機能の有効化 ---
    let enable_forbidden = ask_yes_no("禁止ワードの有効化は行いますか？ (y/n): ")?;
    write_env("FORBIDDENWORDS", if enable_forbidden { "true" } else { "false" })?;

    // --- Step 3: 禁止ワードリストのダウンロード (yを選択した場合のみ) ---
    if enable_forbidden {
        let download_default = ask_yes_no("デフォルト禁止ワードリストをダウンロードしますか？ (y/n): ")?;
        if download_default {
            download_forbidden_words_list().await?;
        }
    }

    // --- Step 4: config.json を書き出す ---
    let config = Config {
        forbidden_words_enabled: enable_forbidden,
    };
    write_config(&config)?;

    Ok(())
}

// ─────────────────────────────────────────────
// ヘルパー関数
// ─────────────────────────────────────────────

/// コンソールに prompt を表示し、1行入力を受け取る
fn prompt(message: &str) -> Result<String, io::Error> {
    print!("{}", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// y / n の入力を受け取り bool を返す。不正入力は再入力を促す
fn ask_yes_no(message: &str) -> Result<bool, io::Error> {
    loop {
        let answer = prompt(message)?;
        match answer.to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no"  => return Ok(false),
            _ => println!("「y」か「n」を入力してください。"),
        }
    }
}

/// .env ファイルに KEY=VALUE 形式で追記 (既存エントリは上書き)
fn write_env(key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    // 既存の .env を読み込み、該当キーを除外して再構築する
    let existing = if std::path::Path::new(".env").exists() {
        fs::read_to_string(".env")?
    } else {
        String::new()
    };

    let mut lines: Vec<String> = existing
        .lines()
        .filter(|l| !l.starts_with(&format!("{}=", key)))
        .map(|l| l.to_string())
        .collect();

    lines.push(format!("{}={}", key, value));
    fs::write(".env", lines.join("\n") + "\n")?;
    Ok(())
}

/// config.json を書き出す
fn write_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(config)?;
    std::fs::write("config.json", json)?;
    println!("config.json を保存しました。");
    Ok(())
}

/// デフォルト禁止ワードリストを JSON でダウンロードしてプロジェクトルートに保存する
async fn download_forbidden_words_list() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: 実際のダウンロード先 URL に差し替えてください
    const DOWNLOAD_URL: &str = "https://example.com/forbidden_words.json";

    println!("禁止ワードリストをダウンロード中... ({})", DOWNLOAD_URL);

    let response = reqwest::get(DOWNLOAD_URL).await?;

    if !response.status().is_success() {
        return Err(format!(
            "ダウンロードに失敗しました: HTTP {}",
            response.status()
        )
        .into());
    }

    let bytes = response.bytes().await?;
    std::fs::write("forbidden_words.json", &bytes)?;

    println!("forbidden_words.json を保存しました。");
    Ok(())
}