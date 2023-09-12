use crate::structs::entry::EntryRecruiter;
use crate::structs::entry::EntryRegular;
use crate::structs::entry::EntryScrapConnection;
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use serde_json::json;
use tracing::{debug, error, info};

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
use crate::actions::withdraw_connection::withdraw;
use structs::entry::Entry;
use structs::entry::EntryScrapSearchRecruiter;
use structs::entry::EntryScrapSearchRegular;
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

    let _spawn = task::spawn(async move {
        let api = scrap(json.into_inner());
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

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Scraping Conversations started!"
    }))
}
#[tracing::instrument]
#[post("/scrap_inmails")]
async fn scrap_inmails_conversations(json: web::Json<EntryRecruiter>) -> HttpResponse {
    info!("This is some additional information");
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();

    let _spawn = task::spawn(async move {
        let api = scrap_inmails(json.into_inner());
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

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Scraping Inmails started!"
    }))
}

#[post("/scrap_connection")]
async fn scrap_connection(json: web::Json<EntryScrapConnection>) -> HttpResponse {
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    tokio::spawn(async move {
        let api = scrap_connections(json.into_inner());
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

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Scraping Connections started!"
    }))
}

#[post("/scrap_regular_search")]
async fn scrap_regular_search_url(json: web::Json<EntryScrapSearchRegular>) -> HttpResponse {
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    let aisearch = json.aisearch.clone();
    tokio::spawn(async move {
        let api = scrap_regular_search(json.into_inner());
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

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Scraping of regular search started!"
    }))
}

#[post("/scrap_recruiter_search")]
async fn scrap_recruiter_search_url(json: web::Json<EntryScrapSearchRecruiter>) -> HttpResponse {
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    let aisearch = json.aisearch.clone();
    tokio::spawn(async move {
        let api = scrap_recruiter_search(json.into_inner());
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

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Scraping of recruiter search started!"
    }))
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

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Withdraw Connection started!"
    }))
}

#[post("/message")]
async fn message(json: web::Json<EntrySendConnection>) -> HttpResponse {
    let message_id = json.message_id.clone();
    let message_id_2 = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    let result = tokio::spawn(async move {
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
    tokio::spawn(async move {
        check_task(result, message_id_2).await;
    });
    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Sending Message started!"
    }))
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

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Scraping profile started!"
    }))
}

#[post("/connect")]
async fn connect(json: web::Json<EntrySendConnection>) -> HttpResponse {
    info!("Send connection started {}", json.message_id);
    let message_id = json.message_id.clone();
    let message_id_2 = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();
    let result = tokio::spawn(async move {
        match connection(json.into_inner()).await {
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
    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Scraping profile started!"
    }))
}

#[post("/send_inmail")]
async fn send_inmail(json: web::Json<EntrySendInmail>) -> HttpResponse {
    info!("Send Inmail started {}", json.message_id);
    let message_id = json.message_id.clone();
    let message_id_2 = json.message_id.clone();
    let webhook = json.webhook.clone();
    let user_id = json.user_id.clone();

    let result: task::JoinHandle<()> = tokio::spawn(async move {
        let api = send_inmails(json.into_inner());

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
    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Sending Inmail started!"
    }))
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
            .service(scrap_regular_search_url)
            .service(scrap_recruiter_search_url)
    })
    .bind(address)?
    .run()
    .await
}

async fn check_task(task: task::JoinHandle<()>, message_id: String) {
    let webhook = "https://overview.tribe.xyz/api/1.1/wf/checking_thread_task";
    match task.await {
        Ok(_) => info!("Task was finished successfully, {}", message_id),
        Err(error) => {
            debug!(error = ?error, "An error occurred/Task Checked {}", message_id);
            error!(error = ?error, "An error occurred/Task Checked {}", message_id);
            let client = reqwest::Client::new();
            let payload = json!({
                "result": error.to_string(),
                "message_id": message_id,
                "error": "yes",
            });
            let res = client.post(webhook).json(&payload).send().await;
            match res {
                Ok(_) => info!("Http for task, {} was done", message_id),
                Err(error) => {
                    error!(error = ?error, "Http for task {} returned error {}", message_id, error);
                }
            }
        }
    }
}
// To solve {} issues with empty json response
// + 1. add separate thread for awaiting of initial thread and see what is the result there
// + 2. add tracing for error like, debug, warning etc. (look at tracing subsciber)
// 3. switch from actix to axum
// 4. switch to aws or gcp
