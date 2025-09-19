# onjbot

![](https://i.imgur.com/WGVVDcf.png)

## 概要

おんJ民が作った、ちょっと便利なDiscordボットやで。  
記事を引っぱってきたり、ローカルのAIとおしゃべりできたりするんや。  
アイコンは解音ゼロで、見た目もかわいくしといたで。  

## 技術スタック

- Rust製  
- ローカルLLM (text-generation-webui) のAPIを使用  

## コマンド一覧

### スラッシュコマンド

| コマンド名   | 説明                        |
|--------------|-----------------------------|
| `/ping`      | Botが動いているか確認します  |
| `/rss`       | チャンネルのRSSフィードを投稿します |
| `/rss-random`| ランダムにRSSフィードを投稿します |
| `/gen`       | 画像生成します |
| `/zenres`    | 全てのメッセージに反応するモードに切り替えます |

### 通常コマンド

- `!ai` — Botと会話できます（内部でローカルLLMのtext-generation-webui APIを使用）

### うんJ連携機能

| コマンド名   | 説明                        |
|--------------|-----------------------------|
| `!beep`      | Botが動いているか確認します  |
| `!ai`        | Botと会話できます |
| `!gen`       | 画像生成します |

## 起動コマンド

### onjbot

```sh
cargo build
cargo run
```

### ai

```
./start_windows.bat --listen --listen-host 127.0.0.11
```


```
./start_windows.bat --listen --listen-host 127.0.0.11 --api --api-port 5001
```

### gen

webui-user.bat

```
set COMMANDLINE_ARGS=--listen --server-name 127.0.0.12 --api
```

```
./webui-user.bat
```

### feeder

※PowerShell
※HttpOnly混合
※ルームに戻る & 設定変更

```
$env:FEEDER_COOKIE='クッキーの値'
```

### unj

※本番に向いている

## ライセンス

- AGPL-3.0