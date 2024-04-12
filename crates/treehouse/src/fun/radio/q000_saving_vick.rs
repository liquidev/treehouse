use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State,
    },
    http::header::SET_COOKIE,
    response::{sse::Event, IntoResponse, Response, Sse},
};
use chrono::{DateTime, Datelike, Utc};
use tokio::sync::{mpsc::unbounded_channel, watch};
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::{
    authentication::{ProgressCookie, NO_PROGRESS},
    questline::{self, Q000_SAVING_VICK},
    RadioState,
};

pub struct QuestState {
    sos_message_rx: watch::Receiver<String>,
}

impl QuestState {
    const DOOR_OPENED: u8 = b'1';

    pub fn new() -> Self {
        let (sos_message_tx, sos_message_rx) = watch::channel(String::new());
        tokio::spawn(sos_task(sos_message_tx));

        Self { sos_message_rx }
    }
}

// Resource Constrained Radio messages.
async fn send_rcr_message(tx: &watch::Sender<String>, message: &str) {
    tokio::time::sleep(Duration::from_secs(message.len() as u64)).await;
    _ = tx.send(String::from(message));
}

pub async fn sos_task(tx: watch::Sender<String>) {
    loop {
        send_rcr_message(&tx, "DEVICE - SOS").await;
        send_rcr_message(&tx, "HELP").await;
        send_rcr_message(&tx, "THIS IS VICK").await;
        send_rcr_message(&tx, "INSIDE THE SANDBOX ROOM").await;
        send_rcr_message(&tx, "I AM TRAPPED").await;
        send_rcr_message(&tx, "THE MAZES WENT INSANE").await;
        send_rcr_message(&tx, "READ URL SLASH SANDBOX SLASH MAINTENANCE").await;
    }
}

pub async fn get_sos(state: State<Arc<RadioState>>) -> impl IntoResponse {
    let (tx, rx) = unbounded_channel::<anyhow::Result<Event>>();
    let mut sos_message_rx = state.q000.sos_message_rx.clone();
    sos_message_rx.borrow_and_update();
    tokio::spawn(async move {
        _ = tx.send(Ok(
            Event::default().data("/treehouse/protocol/resource-constrained-radio/v1")
        ));

        loop {
            _ = sos_message_rx.changed().await;
            let value = sos_message_rx.borrow().clone();
            _ = tx.send(Ok(Event::default().data(value)));
        }
    });
    Sse::new(UnboundedReceiverStream::new(rx))
}

pub async fn get_door_status(progress: ProgressCookie) -> impl IntoResponse {
    if progress.get(questline::Q000_SAVING_VICK) >= QuestState::DOOR_OPENED {
        "open"
    } else {
        "closed"
    }
}

pub async fn door_control(ws: WebSocket) {
    async fn ws_loop(mut ws: WebSocket) -> anyhow::Result<()> {
        ws.send(Message::Text("/treehouse/protocol/sandbox-door/v1".into()))
            .await?;

        Ok(())
    }

    _ = ws_loop(ws);
}
