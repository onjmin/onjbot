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

### 通常コマンド

- `!ai` — Botと会話できます（内部でローカルLLMのtext-generation-webui APIを使用）

### 起動コマンド

```sh
cargo build
cargo run
```

```
.\start_windows.bat --api
```

## ライセンス

- AGPL-3.0