mod authentication;
mod key_alpha;
mod q000_saving_vick;

use std::{sync::Arc, time::Duration};

use axum::{
    response::{sse::Event, IntoResponse, Sse},
    routing::{get, post},
    Router,
};
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;

use self::key_alpha::AlphaKeys;

mod questline {
    pub const Q000_SAVING_VICK: usize = 0;
}

struct RadioState {
    alpha_keys: Arc<AlphaKeys>,
    q000: q000_saving_vick::QuestState,
}

pub fn radio<S>() -> Router<S> {
    let alpha_keys = Arc::new(AlphaKeys::new());
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
        .route("/station/1395024452", post(q000_saving_vick::door_control))
        .nest(
            "/station/1801812339",
            key_alpha::router(Arc::clone(&alpha_keys)),
        )
        .with_state(Arc::new(RadioState {
            alpha_keys,
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
            protocol: &'static str,
            // Different frequencies!! Every station should have a different heartbeat.
            heartbeat: Duration,
        }

        let radio_stations = [RadioStation {
            path: "/radio/station/1397838624",
            protocol: "/treehouse/protocol/resource-constrained-radio/v1",
            heartbeat: Duration::from_millis(12345),
        }];

        for station in radio_stations {
            let tx = tx.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(station.heartbeat).await;
                    _ = tx.send(Ok(Event::default()
                        .data(format!("station {} {}", station.path, station.protocol))));
                }
            });
        }
    });
    Sse::new(UnboundedReceiverStream::new(rx))
}
