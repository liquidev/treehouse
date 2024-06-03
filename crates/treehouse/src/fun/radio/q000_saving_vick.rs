use std::{sync::Arc, time::Duration};

use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::{sse::Event, IntoResponse, Sse},
};
use tokio::sync::{mpsc::unbounded_channel, watch};
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::{authentication::ProgressCookie, questline, RadioState};

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

pub async fn door_control(
    progress: ProgressCookie,
    radio_state: State<Arc<RadioState>>,
    command: String,
) -> impl IntoResponse {
    if let Some((command, key)) = command.split_once(' ') {
        if !radio_state.alpha_keys.is_valid(key) {
            return (StatusCode::FORBIDDEN, HeaderMap::new(), "error: invalid key. please request an Alpha Key via /treehouse/protocol/key-alpha/v1");
        }
        match command {
            "open" => {
                if progress.get(questline::Q000_SAVING_VICK) >= QuestState::DOOR_OPENED {
                    (
                        StatusCode::FORBIDDEN,
                        HeaderMap::new(),
                        "error: door already open",
                    )
                } else {
                    let mut progress = progress;
                    progress.set(questline::Q000_SAVING_VICK, QuestState::DOOR_OPENED);
                    (
                        StatusCode::OK,
                        HeaderMap::from_iter([(SET_COOKIE, progress.to_header_value())]),
                        "ok",
                    )
                }
            }
            "close" => {
                if progress.get(questline::Q000_SAVING_VICK) >= QuestState::DOOR_OPENED {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        HeaderMap::new(),
                        "error: door stuck",
                    )
                } else {
                    (
                        StatusCode::FORBIDDEN,
                        HeaderMap::new(),
                        "error: door already closed",
                    )
                }
            }
            _ => (
                StatusCode::BAD_REQUEST,
                HeaderMap::new(),
                "error: unknown command. available commands: `open`, `close`",
            ),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            HeaderMap::new(),
            "error: command must take the form `COMMAND KEY`",
        )
    }
}