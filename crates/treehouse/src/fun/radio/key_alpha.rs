use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use dashmap::{DashMap, DashSet};
use rand::prelude::SliceRandom;
use rand::{random, thread_rng};
use wyrand::WyRand;

use crate::fun::maze::Maze;

pub struct AlphaKeys {
    keys: DashSet<String>,
}

impl AlphaKeys {
    const TIMEOUT: Duration = Duration::from_secs(30);

    pub fn new() -> Self {
        Self {
            keys: DashSet::new(),
        }
    }

    fn generate(self: Arc<Self>) -> String {
        let mut key = String::new();
        let mut rng = thread_rng();
        for _ in 0..=32 {
            key.push(*b"abcdefghijklmnopqrstuvwxyz".choose(&mut rng).unwrap() as char);
        }
        key.push('\u{03b1}');

        self.keys.insert(key.clone());

        let key2 = key.clone();
        // Probably not that efficient to have many small tasks just for deleting keys; a better
        // solution would be a background thread that handles deletion of stale keys, but this is
        // good enough for now.
        tokio::spawn(async move {
            tokio::time::sleep(Self::TIMEOUT).await;
            self.keys.remove(&key2);
        });

        key
    }

    pub fn is_valid(&self, key: &str) -> bool {
        self.keys.contains(key)
    }
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(key_registrar)
}

async fn key_registrar(ws: WebSocket) {
    async fn fallible(mut ws: WebSocket) -> anyhow::Result<()> {
        ws.send(Message::Text("/treehouse/protocol/key-alpha/v1".into()))
            .await?;

        while let Some(message) = ws.recv().await {
            let message = message?;
            if message.to_text().is_ok_and(|c| c == "key") {
                let mut maze = Maze::new(11, 11);
                let mut rng = WyRand::new(random());
                maze.generate(&mut rng);
                ws.send(Message::Text(maze.render_default())).await?;
            } else {
                ws.send(Message::Text("error: unknown command".into()))
                    .await?;
            }
        }
        ws.close().await?;

        Ok(())
    }

    _ = fallible(ws).await;
}

pub fn router<S>() -> Router<S> {
    Router::new()
        .route("/", get(handler))
        .with_state(Arc::new(AlphaKeys::new()))
}
