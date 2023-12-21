use crate::structs::entry::EntryRecruiter;
use crate::structs::entry::EntryRegular;
use crate::structs::entry::EntryScrapConnection;
use serde_json::json;
use tracing::{debug, error, info};

use axum::routing::post;
use axum::{response::IntoResponse, Json, Router};
use tokio::task;

use std::net::SocketAddr;
mod actions;
mod structs;
use crate::actions::connection::connection;
use crate::actions::scrap_connections::scrap_connections;
use crate::actions::scrap_conversations::scrap;
use crate::actions::scrap_inmails::scrap_inmails;
use crate::actions::scrap_profile::scrap_profile;
use crate::actions::scrap_recruiter_search::scrap_recruiter_search;
use crate::actions::scrap_regular_search::scrap_regular_search;
use crate::actions::send_inmails::send_inmails;
use crate::actions::send_message::send_message;
use crate::actions::serialize::serialize_json;
use crate::actions::withdraw_pending_connection::withdraw_pending;
use structs::entry::EntryScrapSearchRecruiter;
use structs::entry::EntryScrapSearchRegular;
use structs::entry::EntrySendConnection;
use structs::entry::EntrySendInmail;
use structs::entry::PhantomGetJson;

async fn serialize(json: Json<PhantomGetJson>) -> impl IntoResponse {
    let _spawn = task::spawn(async move {
        let api = serialize_json(json.0);
        match api.await {
            Ok(_) => info!("Serialization was successful!"),
            Err(error) => error!("Serialization error: {}", error),
        }
    });

    Json(json!({
        "status": "success",
        "message": "Serialization started!"
    }))
}
async fn scrap_conversations(json: Json<EntryRegular>) -> impl IntoResponse {
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();

    let _spawn = task::spawn(async move {
        let api = scrap(json.0);
        match api.await {
            Ok(_) => info!("Scraping messages was successful!"),
            Err(error) => {
                let client = reqwest::Client::new();
                let payload = json!({
                    "result": error.to_string(),
                    "user_id": user_id,
                    "error": "yes",
                });
                let _res = client.post(webhook).json(&payload).send().await;
            }
        }
    });

    Json(json!({
        "status": "success",
        "message": "Scarping conversations started!"
    }))
}

async fn scrap_inmails_conversations(json: Json<EntryRecruiter>) -> impl IntoResponse {
    info!("This is some additional information");
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();

    let _spawn = task::spawn(async move {
        let api = scrap_inmails(json.0);
        match api.await {
            Ok(_) => info!("Scraping messages was successful!"),
            Err(error) => {
                let client = reqwest::Client::new();
                let payload = json!({
                    "result": error.to_string(),
                    "user_id": user_id,
                    "error": "yes",
                });
                let _res = client.post(webhook).json(&payload).send().await;
            }
        }
    });

    Json(json!({
        "status": "success",
        "message": "Scrap Inmails started!"
    }))
}

async fn scrap_connection(json: Json<EntryScrapConnection>) -> impl IntoResponse {
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    tokio::spawn(async move {
        let api = scrap_connections(json.0);
        match api.await {
            Ok(_) => info!("Scraping connections was successful!"),
            Err(error) => {
                let client = reqwest::Client::new();
                let payload = json!({
                    "result": error.to_string(),
                    "user_id": user_id,
                    "error": "yes",
                });
                let _res = client.post(webhook).json(&payload).send().await;
            }
        }
    });

    Json(json!({
        "status": "success",
        "message": "Scrap connection started!"
    }))
}

async fn scrap_regular_search_url(json: Json<EntryScrapSearchRegular>) -> impl IntoResponse {
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    let aisearch = json.aisearch.clone();
    tokio::spawn(async move {
        let api = scrap_regular_search(json.0);
        match api.await {
            Ok(_) => info!("Scraping regular search was successful!"),
            Err(error) => {
                let client = reqwest::Client::new();
                let payload = json!({
                    "result": error.to_string(),
                    "aisearch": aisearch,
                    "user_id": user_id,
                    "error": "yes",
                });
                let _res = client.post(webhook).json(&payload).send().await;
            }
        }
    });

    Json(json!({
        "status": "success",
        "message": "Scrap regular search started!"
    }))
}

async fn scrap_recruiter_search_url(json: Json<EntryScrapSearchRecruiter>) -> impl IntoResponse {
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    let aisearch = json.aisearch.clone();
    tokio::spawn(async move {
        let api = scrap_recruiter_search(json.0);
        match api.await {
            Ok(_) => info!("Scraping recruiter search was successful!"),
            Err(error) => {
                let client = reqwest::Client::new();
                let payload = json!({
                    "result": error.to_string(),
                    "aisearch": aisearch,
                    "user_id": user_id,
                    "error": "yes",
                });
                let _res = client.post(webhook).json(&payload).send().await;
            }
        }
    });

    Json(json!({
        "status": "success",
        "message": "Scrap recruiter search started!"
    }))
}

async fn withdraw_connection(json: Json<EntrySendConnection>) -> impl IntoResponse {
    let message_id = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    tokio::spawn(async move {
        let api = withdraw_pending(json.0);
        match api.await {
            Ok(_) => {
                let client = reqwest::Client::new();
                let payload = json!({
                    "message": message_id,
                    "result": "Connection was withdrawn",
                    "user_id": user_id,
                    "error": "no",
                });
                let _res = client.post(webhook).json(&payload).send().await;
            }
            Err(error) => {
                let client = reqwest::Client::new();
                let payload = json!({
                    "message": message_id,
                    "result": error.to_string(),
                    "user_id": user_id,
                    "error": "yes",
                });
                let _res = client.post(webhook).json(&payload).send().await;
            }
        }
    });

    Json(json!({
        "status": "success",
        "message": "Withdraw connection started!"
    }))
}

async fn message(json: Json<EntrySendConnection>) -> impl IntoResponse {
    let message_id = json.message_id.clone();
    let message_id_2 = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    let result = tokio::spawn(async move {
        let api = send_message(json.0);
        match api.await {
            Ok(_) => {
                let client = reqwest::Client::new();
                let payload = json!({
                    "message": message_id,
                    "result": "Message was sent",
                    "user_id": user_id,
                    "error": "no",
                });
                let _res = client.post(webhook).json(&payload).send().await;
            }
            Err(error) => {
                let client = reqwest::Client::new();
                let payload = json!({
                    "message": message_id,
                    "result": error.to_string(),
                    "user_id": user_id,
                    "error": "yes",
                });
                let _res = client.post(webhook).json(&payload).send().await;
            }
        }
    });
    tokio::spawn(async move {
        check_task(result, message_id_2).await;
    });
    Json(json!({
        "status": "success",
        "message": "Sending message started!"
    }))
}

async fn scrap_profiles(json: Json<EntrySendConnection>) -> impl IntoResponse {
    let message_id = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    tokio::spawn(async move {
        let api = scrap_profile(json.0);
        match api.await {
            Ok(_) => {
                let client = reqwest::Client::new();
                let payload = json!({
                    "message": message_id,
                    "result": "Profile was scrapped",
                    "user_id": user_id,
                    "error": "no",
                });
                let _res = client.post(webhook).json(&payload).send().await;
            }
            Err(error) => {
                let client = reqwest::Client::new();
                let payload = json!({
                    "message": message_id,
                    "result": error.to_string(),
                    "user_id": user_id,
                    "error": "yes",
                });
                let _res = client.post(webhook).json(&payload).send().await;
            }
        }
    });

    Json(json!({
        "status": "success",
        "message": "Scrap profile started!"
    }))
}

async fn connect(json: Json<EntrySendConnection>) -> impl IntoResponse {
    info!("Send connection started {}", json.message_id);
    let message_id = json.message_id.clone();
    let message_id_2 = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    let result = tokio::spawn(async move {
        match connection(json.0).await {
            Ok(_) => {
                info!("Connection sent successfully {}", message_id);
                let client = reqwest::Client::new();
                let payload = json!({
                    "message": message_id,
                    "result": "Connection was sent",
                    "user_id": user_id,
                    "error": "no",
                });
                let _res = client.post(webhook).json(&payload).send().await;
            }
            Err(error) => {
                error!("Error sending connection for {}: {}", message_id, error);
                let client = reqwest::Client::new();
                let payload = json!({
                    "message": message_id,
                    "result": error.to_string(),
                    "user_id": user_id,
                    "error": "yes",
                });
                let res = client.post(webhook).json(&payload).send().await;
                match res {
                    Ok(_) => info!("Connection http for message, {} was done", message_id),
                    Err(error) => {
                        error!(error = ?error, "Connection http for message {} returned error {}", message_id, error);
                    }
                }
            }
        }
    });
    tokio::spawn(async move {
        check_task(result, message_id_2).await;
    });
    Json(json!({
        "status": "success",
        "message": "Sending connection started!"
    }))
}

async fn send_inmail(json: Json<EntrySendInmail>) -> impl IntoResponse {
    info!("Send Inmail started {}", json.message_id);
    let message_id = json.message_id.clone();
    let message_id_2 = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();

    let result: task::JoinHandle<()> = tokio::spawn(async move {
        let api = send_inmails(json.0);

        match api.await {
            Ok(_) => {
                info!("Send Inmail finished succesfully {}", message_id);
                let client = reqwest::Client::new();
                let payload = json!({
                    "message": message_id,
                    "result": "Inmail was sent",
                    "user_id": user_id,
                    "error": "no",
                });
                let res = client.post(webhook).json(&payload).send().await;
                match res {
                    Ok(_) => info!("Inmail http for message/Ok, {} was done", message_id),
                    Err(error) => {
                        error!(error = ?error, "Inmail http/Ok for message {} returned error {}", message_id, error);
                    }
                }
            }
            Err(error) => {
                error!(error = ?error, "An error occurred/Imail sent {}, error {}", message_id, error);
                let client = reqwest::Client::new();
                let payload = json!({
                    "message": message_id,
                    "result": error.to_string(),
                    "user_id": user_id,
                    "error": "yes",
                });
                let res = client.post(webhook).json(&payload).send().await;
                match res {
                    Ok(_) => info!("Inmail http for message/Error, {} was done", message_id),
                    Err(error) => {
                        error!(error = ?error, "Inmail http for message?error {} returned error {}", message_id, error);
                    }
                }
            }
        }
    });

    tokio::spawn(async move {
        check_task(result, message_id_2).await;
    });
    Json(json!({
        "status": "success",
        "message": "Sending Inmail started!"
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let port = match std::env::var("PORT") {
        Ok(val) => val,
        Err(_e) => "8080".to_string(),
    };
    let address: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Failed to parse address");

    let app = Router::new()
        .route("/send_inmail", post(send_inmail))
        .route("/connect", post(connect))
        .route("/scrap_conversations", post(scrap_conversations))
        .route("/message", post(message))
        .route("/withdraw_connection", post(withdraw_connection))
        .route("/scrap_connection", post(scrap_connection))
        .route("/scrap_inmails", post(scrap_inmails_conversations))
        .route("/scrap_profiles", post(scrap_profiles))
        .route("/scrap_regular_search", post(scrap_regular_search_url))
        .route("/serialize", post(serialize))
        .route("/scrap_recruiter_search", post(scrap_recruiter_search_url));

    hyper::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .expect("Server failed");
}

async fn check_task(task: task::JoinHandle<()>, message_id: String) {
    let webhook = "https://overview.tribe.xyz/api/1.1/wf/checking_thread_task";
    match task.await {
        Ok(_) => info!("Task was finished successfully"),
        Err(error) => {
            debug!(error = ?error, "An error occurred/Task Checked {}, error {}", message_id, error);
            error!(error = ?error, "An error occurred/Task Checked {}, error {}", message_id, error);
            let client = reqwest::Client::new();
            let payload = json!({
                "result": error.to_string(),
                "message_id": message_id,
                "error": "yes",
            });
            let _res = client.post(webhook).json(&payload).send().await;
        }
    }
}
// To solve {} issues with empty json response
// + 1. add separate thread for awaiting of initial thread and see what is the result there
// 2. add tracing for error like, debug, warning etc. (look at tracing subsciber)
// 3. + switch from actix to axum
// 4. switch to aws or gcp
