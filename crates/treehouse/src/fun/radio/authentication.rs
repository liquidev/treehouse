use std::{convert::Infallible, str::FromStr};

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{
        header::{COOKIE, SET_COOKIE},
        request::Parts,
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
};

pub async fn init_progress_cookie(headers: HeaderMap) -> impl IntoResponse {
    for cookie in headers.get_all(COOKIE) {
        if cookie.to_str().is_ok_and(|s| s.contains("progress=")) {
            return (HeaderMap::new(), "You are already authorized.");
        }
    }

    (
        HeaderMap::from_iter([(
            SET_COOKIE,
            ProgressCookie::new().to_header_value(),
        )]),
        "CONGRULATIONS\n-------------\nThank you for agreeing to the treehouse progress cookie policy.\nYou are now authorized personnel.",
    )
}

pub const NO_PROGRESS: u8 = b'0';

pub struct ProgressCookie {
    questlines: [u8; Self::QUESTLINES],
}

impl ProgressCookie {
    const QUESTLINES: usize = 32;

    pub fn new() -> Self {
        Self {
            questlines: [NO_PROGRESS; Self::QUESTLINES],
        }
    }

    fn is_valid_progress_character(character: u8) -> bool {
        character.is_ascii_digit()
    }

    pub fn set(&mut self, questline: usize, progress: u8) {
        assert!(
            questline < Self::QUESTLINES,
            "max {} questlines are currently allowed",
            Self::QUESTLINES
        );
        assert!(
            Self::is_valid_progress_character(progress),
            "invalid progress character"
        );

        self.questlines[questline] = progress;
    }

    pub fn get(&self, questline: usize) -> u8 {
        self.questlines[questline]
    }

    pub fn to_header_value(&self) -> HeaderValue {
        let mut s = String::from("progress=");
        s.reserve(Self::QUESTLINES);
        s.push_str("; Max-Age=34560000; Path=/; SameSite=Lax");
        HeaderValue::from_str(&s).expect("progress cookie header contains invalid characters")
    }
}

impl FromStr for ProgressCookie {
    // NOTE: Infallible because invalid characters are ignored.
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cookie = ProgressCookie::new();

        for i in 0..s.len().min(Self::QUESTLINES) {
            let b = s.as_bytes()[i];
            if Self::is_valid_progress_character(b) {
                cookie.questlines[i] = b;
            }
        }

        Ok(cookie)
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for ProgressCookie {
    type Rejection = NoProgressCookie;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        if let Some(cookies) = parts.headers.get(COOKIE) {
            for cookie in cookies.as_bytes().split(|&c| c == b';') {
                if let Ok(cookie) = std::str::from_utf8(cookie) {
                    if let Some(progress) = cookie.strip_prefix("progress=") {
                        return Ok(ProgressCookie::from_str(progress).unwrap());
                    }
                }
            }
        }
        Err(NoProgressCookie)
    }
}

pub struct NoProgressCookie;

impl IntoResponse for NoProgressCookie {
    fn into_response(self) -> Response {
        (StatusCode::FORBIDDEN, "/treehouse/protocol/authentication/v1\nAccess denied. Please create an account before proceeding.").into_response()
    }
}
