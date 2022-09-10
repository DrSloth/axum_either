use std::net::{SocketAddr, TcpListener as StdTcpListener};
use tokio::net::TcpListener;

use axum::{Form, Json, Router};
use axum_either::AxumEither;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 8080))).await?;
    run(listener.into_std()?).await
}

pub async fn run(listener: StdTcpListener) -> anyhow::Result<()> {
    use axum::routing::post;

    // Setup the router with our routes
    let router = Router::new()
        .route("/get_request_type", post(get_request_type))
        .route("/as_string", post(as_string))
        .route("/echo", post(echo));

    axum::Server::from_tcp(listener)?
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

pub async fn get_request_type(
    request: axum_either::one_of!(Json<Request>, Form<Request>, String),
) -> &'static str {
    axum_either::match_one_of! {request,
        _ => "Json",
        _ => "Form",
        _ => "String",
    }
}

pub async fn echo(
    request: axum_either::one_of!(Json<Request>, Form<Request>, String),
) -> axum_either::one_of!(Json<Request>, Form<Request>, String) {
    let either = axum_either::map_one_of! {request,
        Json(j) => Json(j),
        f => f,
        s => s,
    };
    either
}

pub async fn as_string(
    request: axum_either::one_of!(Json<Request>, Form<Request>, String),
) -> String {
    let resp = axum_either::match_one_of! {request,
        Json(j) => format!("Json: {:?}", j),
        f => format!("Format: {:?}", f),
        s => format!("String: {:?}", s),
    };
    resp
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub id: u32,
}
