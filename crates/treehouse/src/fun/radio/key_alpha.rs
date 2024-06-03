use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use anyhow::bail;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use dashmap::DashSet;
use rand::prelude::SliceRandom;
use rand::{random, thread_rng};
use wyrand::WyRand;

use crate::fun::maze::{Cell, Maze};

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
            key.push(*b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".choose(&mut rng).unwrap() as char);
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

async fn handler(ws: WebSocketUpgrade, State(alpha_keys): State<Arc<AlphaKeys>>) -> Response {
    ws.on_upgrade(move |ws| async move {
        key_registrar(ws, alpha_keys).await;
    })
}

fn check_maze_solution(maze: &Maze, solution: &str) -> anyhow::Result<()> {
    struct Hamster<'a> {
        maze: &'a Maze,
        path: HashSet<(u32, u32)>,
        x: u32,
        y: u32,
    }

    impl<'a> Hamster<'a> {
        fn visualize_path(&self) -> String {
            self.maze.render(|x, y, c| match c {
                Cell::Empty => {
                    if self.path.contains(&(x, y)) {
                        ".."
                    } else {
                        "  "
                    }
                }
                Cell::Solid => {
                    if self.path.contains(&(x, y)) {
                        "!!"
                    } else {
                        "##"
                    }
                }
            })
        }

        fn step(&mut self, dx: i32, dy: i32) -> anyhow::Result<()> {
            self.x = self.x.saturating_add_signed(dx);
            self.y = self.y.saturating_add_signed(dy);
            if !self.path.insert((self.x, self.y)) {
                bail!("the hamster backtracked.\nnote: the rules say that the hamster must not backtrack.\n      try to make your hamster smart enough to solve the maze without having to go backwards\n\nhere's the path the hamster took up to the point of error:\n{}", self.visualize_path());
            }
            if self.maze.get((self.x, self.y)).unwrap_or(Cell::Solid) == Cell::Solid {
                bail!("the hamster walked into a wall.\nnote: the walls stretch high enough that the hamster cannot climb over them!\n      try making the hamster solve the maze a bit more cleverly\n\nhere's the path the hamster took up to the point of error:\n{}", self.visualize_path());
            }
            Ok(())
        }
    }

    let mut hamster = Hamster {
        maze,
        path: HashSet::from_iter([(1, 1)]),
        x: 1,
        y: 1,
    };

    for (i, c) in solution.chars().enumerate() {
        match c {
            'E' => {
                hamster.step(1, 0)?;
                hamster.step(1, 0)?;
            }
            'S' => {
                hamster.step(0, 1)?;
                hamster.step(0, 1)?;
            }
            'W' => {
                hamster.step(-1, 0)?;
                hamster.step(-1, 0)?;
            }
            'N' => {
                hamster.step(0, -1)?;
                hamster.step(0, -1)?;
            }
            _ => bail!("invalid character encountered at input string index {i}: {c:?}. only 'E', 'S', 'W', 'N' are allowed")
        }
    }

    if hamster.x != maze.width - 2 && hamster.y != maze.height - 2 {
        bail!("the hamster did not reach the southeastmost corner of the maze")
    }

    Ok(())
}

async fn key_registrar(ws: WebSocket, alpha_keys: Arc<AlphaKeys>) {
    async fn fallible(mut ws: WebSocket, alpha_keys: Arc<AlphaKeys>) -> anyhow::Result<()> {
        ws.send(Message::Text("/treehouse/protocol/key-alpha/v1".into()))
            .await?;

        while let Some(message) = ws.recv().await {
            let message = message?;
            if message.to_text().is_ok_and(|c| c == "key") {
                let mut maze = Maze::new(11, 11 | 1024);
                let mut rng = WyRand::new(random());
                maze.generate(&mut rng);
                ws.send(Message::Text(maze.render_default())).await?;

                let Some(answer) = ws.recv().await else {
                    break;
                };
                let answer = answer?.into_text()?;
                match check_maze_solution(&maze, &answer) {
                    Ok(_) => {
                        ws.send(Message::Text(format!(
                            "ok: {}",
                            alpha_keys.clone().generate()
                        )))
                        .await?;
                    }
                    Err(error) => {
                        ws.send(Message::Text(format!("error: {error}"))).await?;
                    }
                }
            } else {
                ws.send(Message::Text("error: unknown command".into()))
                    .await?;
                break;
            }
        }
        ws.close().await?;

        Ok(())
    }

    _ = fallible(ws, alpha_keys).await;
}

pub fn router<S>(alpha_keys: Arc<AlphaKeys>) -> Router<S> {
    Router::new()
        .route("/", get(handler))
        .with_state(alpha_keys)
}
