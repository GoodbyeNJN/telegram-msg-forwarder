#[macro_use]
extern crate rocket;

use std::{env::var, panic, sync::LazyLock, thread};

use anyhow::Context;
use env_logger::Builder;
use log::{error, LevelFilter};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    serde::json::Json,
    Request,
};
use serde::{Deserialize, Serialize};

static AUTH_TOKEN: LazyLock<Vec<String>> = LazyLock::new(|| {
    let token = var("AUTH_TOKEN")
        .context("Env variable `AUTH_TOKEN` is not set.")
        .unwrap();

    token
        .split(' ')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
});
static CHAT_ID: LazyLock<String> = LazyLock::new(|| {
    var("TELEGRAM_CHAT_ID")
        .context("Env variable `TELEGRAM_CHAT_ID` is not set.")
        .unwrap()
});
static URL: LazyLock<String> = LazyLock::new(|| {
    let token = var("TELEGRAM_BOT_TOKEN")
        .context("Env variable `TELEGRAM_BOT_TOKEN` is not set.")
        .unwrap();

    format!("https://api.telegram.org/bot{}/sendMessage", token)
});
static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::new());

struct Auth;

fn is_valid_token(token: &str) -> bool {
    AUTH_TOKEN.iter().any(|t| t == token)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = req
            .headers()
            .get_one("Authorization")
            .map(|s| s.trim_start_matches("Bearer ").to_string());

        match token {
            None => Outcome::Error((
                Status::Unauthorized,
                "Missing Authorization token.".to_string(),
            )),
            Some(token) if is_valid_token(&token) => Outcome::Success(Auth),
            Some(_) => Outcome::Error((
                Status::Unauthorized,
                "Invalid Authorization token.".to_string(),
            )),
        }
    }
}

#[derive(Deserialize)]
struct Payload {
    title: String,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct Message {
    chat_id: String,
    text: String,
    parse_mode: String,
}

#[post("/forward", data = "<payload>")]
async fn forward(payload: Json<Payload>, auth: Result<Auth, String>) -> String {
    if let Err(err) = auth {
        error!("{}", err);
        return err;
    }

    let message = Message {
        chat_id: CHAT_ID.clone(),
        text: format!("*{}*\n{}", payload.title, payload.message),
        parse_mode: "Markdown".to_string(),
    };

    let result = CLIENT
        .post(URL.clone())
        .json(&message)
        .send()
        .await
        .context("Failed to send message to Telegram.");

    match result {
        Ok(response) if response.status().is_success() => "Success".to_string(),
        Ok(response) => format!(
            "Failed to send message to Telegram.\nStatus: `{}`\nBody: `{}`",
            response.status(),
            response.text().await.unwrap()
        ),
        Err(err) => format!("{}", err),
    }
}

#[launch]
fn rocket() -> _ {
    Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_env("LOG_LEVEL")
        .format_target(false)
        .format_timestamp_secs()
        .format_indent(Some(29))
        .init();

    panic::set_hook(Box::new(|info| {
        let msg1 = info.payload().downcast_ref::<&str>().copied();
        let msg2 = info.payload().downcast_ref::<String>().map(String::as_str);
        if let Some(msg) = msg1.or(msg2) {
            error!("{}", msg);
        }

        error!(
            "An unexpected error occurred.\nAt: thread: `{}`, location: `{}`",
            thread::current().name().unwrap_or("unknown"),
            if let Some(loc) = info.location() {
                loc.to_string()
            } else {
                "unknown".to_string()
            }
        );
    }));

    rocket::build().mount("/api", routes![forward])
}
