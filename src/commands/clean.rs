use serenity::all::{
    CommandDataOptionValue, CommandInteraction, Context, CreateInteractionResponseFollowup,
    GetMessages, MessageId, Permissions,
};
use std::collections::{HashMap, HashSet};
use std::env;

const KEEP_COUNT: usize = 4; // 【✅ 残す重複メッセージの数】

pub async fn handle_slash_clean(ctx: &Context, command: &CommandInteraction) {
    if let Err(e) = command.defer_ephemeral(&ctx.http).await {
        eprintln!("defer エラー: {:?}", e);
        return;
    }

    // モデレーターチェック
    let mod_role_id: u64 = env::var("DISCORD_MODERATOR_ROLE_ID")
        .expect("DISCORD_MODERATOR_ROLE_ID must be set")
        .parse()
        .expect("DISCORD_MODERATOR_ROLE_ID must be a number");
    let mod_role = serenity::all::RoleId::new(mod_role_id);

    let member = match command.member.as_ref() {
        Some(m) => m,
        None => {
            let _ = command
                .create_followup(
                    &ctx.http,
                    (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                        m.content("❌ ユーザー情報を取得できませんでした")
                            .ephemeral(true)
                    })(CreateInteractionResponseFollowup::default()),
                )
                .await;
            return;
        }
    };

    if !member.roles.iter().any(|r| *r == mod_role) {
        let _ = command
            .create_followup(
                &ctx.http,
                (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                    m.content("❌ このコマンドを実行できるのはモデレーターだけです")
                        .ephemeral(true)
                })(CreateInteractionResponseFollowup::default()),
            )
            .await;
        return;
    }

    // --- 既存の権限チェックと引数解析は省略 ---
    let guild_id = match command.guild_id {
        Some(id) => id,
        None => {
            send_followup(
                ctx,
                command,
                "❌ このコマンドはサーバーのチャンネルでのみ実行できます",
            )
            .await;
            return;
        }
    };
    let channel_id = command.channel_id;

    let member = match command.member.as_ref() {
        Some(m) => m,
        None => {
            send_followup(ctx, command, "❌ ユーザー情報を取得できませんでした").await;
            return;
        }
    };

    let manage_messages = Permissions::MANAGE_MESSAGES;
    if !member
        .permissions
        .map_or(false, |p| p.contains(manage_messages))
    {
        send_followup(
            ctx,
            command,
            "❌ このコマンドを実行するには **メッセージの管理** 権限が必要です",
        )
        .await;
        return;
    }

    let current_user_id = ctx.cache.current_user().id;
    let bot_member = match guild_id.member(&ctx.http, current_user_id).await {
        Ok(m) => m,
        Err(_) => {
            send_followup(
                ctx,
                command,
                "❌ Botのサーバーメンバー情報を取得できませんでした",
            )
            .await;
            return;
        }
    };
    // if !bot_member
    //     .permissions
    //     .map_or(false, |p| p.contains(manage_messages))
    // {
    //     send_followup(
    //         ctx,
    //         command,
    //         "❌ Botに **メッセージの管理** 権限がありません",
    //     )
    //     .await;
    //     return;
    // }

    // 引数解析
    let limit = command
        .data
        .options
        .get(0)
        .and_then(|opt| match opt.value {
            CommandDataOptionValue::Integer(i) => Some(i as u64),
            _ => None,
        })
        .unwrap_or(50)
        .min(100);

    // メッセージ履歴の取得
    let messages = match channel_id
        .messages(&ctx.http, GetMessages::new().limit(limit as u8))
        .await
    {
        Ok(m) => m,
        Err(e) => {
            send_followup(
                ctx,
                command,
                &format!("❌ メッセージ履歴の取得に失敗しました: {}", e),
            )
            .await;
            return;
        }
    };

    // 重複メッセージの検出と削除対象の決定
    // キー: (メッセージ内容, 投稿者ID), 値: その重複メッセージIDのリスト
    let mut duplicate_groups: HashMap<(String, String), Vec<MessageId>> = HashMap::new();
    let mut messages_to_delete: HashSet<MessageId> = HashSet::new();
    let bot_user_id = ctx.cache.current_user().id;

    // Discord APIは新しいメッセージから順に返すため、そのまま順に処理する
    for msg in messages.iter() {
        // Bot自身のメッセージ、コマンドメッセージ、ピン留めメッセージは対象外とする
        if msg.author.id == bot_user_id || msg.id == MessageId::from(command.id.get()) || msg.pinned
        {
            continue;
        }

        // メッセージの内容と投稿者IDをキーとするタプル
        let key = (msg.content.clone(), msg.author.id.to_string());

        // キーに対応するメッセージIDのリストを取得し、メッセージIDを追加
        let group = duplicate_groups.entry(key).or_insert_with(Vec::new);
        group.push(msg.id); // 新しいメッセージから順にIDがリストに追加される
    }

    // リストを逆順にし（古いメッセージから順に）、KEEP_COUNT を超えるものを削除対象にする
    for (_, ids) in duplicate_groups.iter_mut() {
        ids.reverse(); // ここで古いメッセージがリストの先頭に来る

        if ids.len() > KEEP_COUNT {
            // リストの KEEP_COUNT + 1 番目以降（つまり9件目以降）を削除対象としてセットに追加
            for id in ids.iter().skip(KEEP_COUNT) {
                messages_to_delete.insert(*id);
            }
        }
    }

    // 削除対象がなければ終了
    let delete_count = messages_to_delete.len();
    if delete_count == 0 {
        send_followup(
            ctx,
            command,
            &format!(
                "✅ 過去 {} 件のメッセージに、重複が {} 件を超えるものは見つかりませんでした。",
                limit, KEEP_COUNT
            ),
        )
        .await;
        return;
    }

    // 削除処理の実行
    let delete_message_ids: Vec<MessageId> = messages_to_delete.into_iter().collect();

    // 一括削除（一度に最大100件まで）
    let result = if delete_message_ids.len() > 1 {
        // 2件以上なら bulk delete
        channel_id
            .delete_messages(&ctx.http, &delete_message_ids)
            .await
    } else if delete_message_ids.len() == 1 {
        // 1件だけなら通常の delete
        let msg_id = delete_message_ids.into_iter().next().unwrap();
        channel_id.delete_message(&ctx.http, msg_id).await
    } else {
        Ok(())
    };

    match result {
        Ok(_) => {
            send_followup(
                ctx,
                command,
                &format!("✅ 過去 {} 件のメッセージから、{} 件を超える重複メッセージを {} 件削除しました。", limit, KEEP_COUNT, delete_count),
            ).await;
        }
        Err(e) => {
            send_followup(
                ctx,
                command,
                &format!("❌ メッセージの削除に失敗しました: {}", e),
            )
            .await;
        }
    }
}

// フォローアップメッセージを送信するヘルパー関数
async fn send_followup(ctx: &Context, command: &CommandInteraction, content: &str) {
    let _ = command
        .create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content(content)
                .ephemeral(true),
        )
        .await;
}
