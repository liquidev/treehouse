mod authentication;
mod key_alpha;
mod q000_saving_vick;

use std::{sync::Arc, time::Duration};

use axum::{
    response::{sse::Event, IntoResponse, Sse},
    routing::get,
    Router,
};
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;

mod questline {
    pub const Q000_SAVING_VICK: usize = 0;
}

struct RadioState {
    q000: q000_saving_vick::QuestState,
}

pub fn radio<S>() -> Router<S> {
    Router::new()
        .route("/", get(index))
        .route(
            "/station/1094927188",
            get(authentication::init_progress_cookie),
        )
        .route("/station/1397838624", get(q000_saving_vick::get_sos))
        .route(
            "/station/1395024484",
            get(q000_saving_vick::get_door_status),
        )
        .nest("/station/1801812339", key_alpha::router())
        .with_state(Arc::new(RadioState {
            q000: q000_saving_vick::QuestState::new(),
        }))
}

async fn index() -> impl IntoResponse {
    let (tx, rx) = unbounded_channel::<anyhow::Result<Event>>();
    tokio::spawn(async move {
        _ = tx.send(Ok(
            Event::default().data("/treehouse/protocol/radio-index/v1")
        ));

        struct RadioStation {
            path: &'static str,
            // Different frequencies!! Every station should have a different heartbeat.
            heartbeat: Duration,
        }

        let radio_stations = [RadioStation {
            path: "/radio/station/1397838624",
            heartbeat: Duration::from_millis(12345),
        }];

        for station in radio_stations {
            let tx = tx.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(station.heartbeat).await;
                    _ = tx.send(Ok(
                        Event::default().data(format!("station {}", station.path))
                    ));
                }
            });
        }
    });
    Sse::new(UnboundedReceiverStream::new(rx))
}
