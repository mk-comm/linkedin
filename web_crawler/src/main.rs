use crate::structs::entry::EntryRecruiter;
use crate::structs::entry::EntryRegular;
use crate::structs::entry::EntryScrapConnection;
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use serde_json::json;
use tracing::{error, info};

mod actions;
mod structs;
use crate::actions::connection::connection;
use crate::actions::scrap_connections::scrap_connections;
use crate::actions::scrap_conversations::scrap;
use crate::actions::scrap_inmails::scrap_inmails;
use crate::actions::scrap_profile::scrap_profile;
use crate::actions::send_inmails::send_inmails;
use crate::actions::send_message::send_message;
use crate::actions::withdraw_connection::withdraw;
use structs::entry::Entry;
use structs::entry::EntrySendConnection;
use structs::entry::EntrySendInmail;
use tokio::task;
#[get("/")]
async fn index() -> String {
    "Route is not available!".to_string()
}
#[post("/scrap_conversations")]
async fn scrap_conversations(json: web::Json<EntryRegular>) -> HttpResponse {
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();

    let _spawn = task::spawn_local(async move {
        let api = scrap(json.into_inner());
        match api.await {
            Ok(_) => println!("Scraping messages was successful!"),
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

    HttpResponse::Ok().body("Scrapping started!")
}
#[tracing::instrument]
#[post("/scrap_inmails")]
async fn scrap_inmails_conversations(json: web::Json<EntryRecruiter>) -> HttpResponse {
    info!("This is some additional information");
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();

    let _spawn = task::spawn_local(async move {
        let api = scrap_inmails(json.into_inner());
        match api.await {
            Ok(_) => println!("Scraping messages was successful!"),
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

    HttpResponse::Ok().body("Scrapping started!")
}

#[post("/scrap_connection")]
async fn scrap_connection(json: web::Json<EntryScrapConnection>) -> HttpResponse {
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    tokio::spawn(async move {
        let api = scrap_connections(json.into_inner());
        match api.await {
            Ok(_) => println!("Scraping connections was successful!"),
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

    HttpResponse::Ok().body("Scraping connections started!")
}

#[post("/withdraw_connection")]
async fn withdraw_connection(json: web::Json<Entry>) -> HttpResponse {
    let message_id = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    tokio::spawn(async move {
        let api = withdraw(json.into_inner());
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

    HttpResponse::Ok().body("Withdraw started!")
}

#[post("/message")]
async fn message(json: web::Json<EntrySendConnection>) -> HttpResponse {
    let message_id = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    tokio::spawn(async move {
        let api = send_message(json.into_inner());
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

    HttpResponse::Ok().body("Sending message started!")
}

#[post("/scrap_profiles")]
async fn scrap_profiles(json: web::Json<Entry>) -> HttpResponse {
    let message_id = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    tokio::spawn(async move {
        let api = scrap_profile(json.into_inner());
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

    HttpResponse::Ok().body("Sending message started!")
}

#[post("/connect")]
async fn connect(json: web::Json<EntrySendConnection>) -> HttpResponse {
    println!("{:?}", json);
    info!("Received request to connect");
    let message_id = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    tokio::spawn(async move {
        match connection(json.into_inner()).await {
            Ok(_) => {
                info!("Connection sent successfully");
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
                error!("Error sending connection: {}", error);
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

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Sending Connections started!"
    }))
}

#[post("/send_inmail")]
async fn send_inmail(json: web::Json<EntrySendInmail>) -> HttpResponse {
    println!("{:?}", json);
    let message_id = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    tokio::spawn(async move {
        let api = send_inmails(json.into_inner());
        match api.await {
            Ok(_) => {
                let client = reqwest::Client::new();
                let payload = json!({
                    "message": message_id,
                    "result": "Inmail was sent",
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

    HttpResponse::Ok().body("Sending Inmail started!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let port = match std::env::var("PORT") {
        Ok(val) => val,
        Err(_e) => "8080".to_string(),
    };
    let address = format!("0.0.0.0:{}", port);
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(connect)
            .service(scrap_conversations)
            .service(message)
            .service(withdraw_connection)
            .service(scrap_connection)
            .service(scrap_inmails_conversations)
            .service(scrap_profiles)
            .service(send_inmail)
    })
    .bind(address)?
    .run()
    .await
}
