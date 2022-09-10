// A complete example with a website which lets you trigger errors, Forms and Json requests.

use std::{
    borrow::Cow,
    net::{SocketAddr, TcpListener as StdTcpListener},
};
use tokio::net::TcpListener;

use axum::{response::Html, Form, Json, Router};
use axum_either::AxumEither;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 8080))).await?;
    run(listener.into_std()?).await
}

pub async fn run(listener: StdTcpListener) -> anyhow::Result<()> {
    use axum::routing::{get, post};

    // Setup the router with our routes
    let router = Router::new()
        .route("/", get(index))
        .route("/hello", post(hello))
        .route("/bye", post(bye));

    axum::Server::from_tcp(listener)?
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

pub async fn index() -> Html<&'static str> {
    Html(include_str!("./web/form_or_json.html"))
}

/// A route which accepts both a form and json and gives an appropriate response
pub async fn hello(
    request: AxumEither<Json<HelloRequest>, Form<HelloRequest>>,
) -> AxumEither<Json<HelloResponse>, String> {
    match request {
        AxumEither::Left(Json(req)) => {
            println!("Got JSON in hello");
            AxumEither::Left(Json(HelloResponse {
                msg: format!("Hello {}!", req.name).into(),
                name: req.name,
            }))
        }
        AxumEither::Right(Form(req)) => {
            println!("Got Form in hello");
            AxumEither::Right(format!("Hi {}!", req.name))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HelloRequest {
    pub name: Cow<'static, str>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct HelloResponse {
    pub msg: Cow<'static, str>,
    pub name: Cow<'static, str>,
}

/// A route which accepts both a Form and Json and gives the same response for both
pub async fn bye(request: AxumEither<Json<ByeRequest>, Form<ByeRequest>>) -> String {
    fn fmt(name: String) -> String {
        format!("Bye, {}!", name)
    }

    println!("Got {:?} in bye", request);

    request
        .map_lr(|Json(req)| fmt(req.name), |Form(req)| fmt(req.name))
        .into_inner()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ByeRequest {
    pub name: String,
}
