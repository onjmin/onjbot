use once_cell::sync::Lazy;
use serenity::all::ChannelId;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::Mutex;

pub static FEEDS: Lazy<Arc<Mutex<Vec<(ChannelId, String)>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(vec![
        (
            ChannelId::new(1396030037362216980),
            "https://qiita.com/tags/python/feed".to_string(),
        ),
        (
            ChannelId::new(1396051371580461148),
            "https://zenn.dev/topics/scratch/feed".to_string(),
        ),
    ]))
});

pub static POSTED_URLS: Lazy<Arc<Mutex<HashSet<String>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));
