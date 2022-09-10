include!("../examples/form_or_json.rs");

async fn test_setup() -> SocketAddr {
    let listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { run(listener).await });
    addr
}

#[tokio::test]
async fn json_hello_responds_with_json() {
    let addr = test_setup().await;
    let client = reqwest::Client::new();

    let tests = [
        (
            "Hassan",
            HelloResponse {
                name: "Hassan".into(),
                msg: "Hello Hassan!".into(),
            },
        ),
        (
            "Kaworu Nagisa",
            HelloResponse {
                name: "Kaworu Nagisa".into(),
                msg: "Hello Kaworu Nagisa!".into(),
            },
        ),
        (
            "Reimu Hakurei",
            HelloResponse {
                name: "Reimu Hakurei".into(),
                msg: "Hello Reimu Hakurei!".into(),
            },
        ),
    ];

    let hello_addr = format!("http://{}/hello", addr);
    for (name, expected_response) in tests {
        let response = client
            .post(&hello_addr)
            .header("Content-Type", "application/json")
            .json(&HelloRequest { name: name.into() })
            .send()
            .await
            .expect("Failed to send test request")
            .json()
            .await
            .expect("Failed to parse response");
        assert_eq!(expected_response, response);
    }
}

#[tokio::test]
async fn form_hello_responds_with_string() {
    let addr = test_setup().await;
    let client = reqwest::Client::new();

    let tests = [
        ("Hassan", "Hi Hassan!"),
        ("Kaworu Nagisa", "Hi Kaworu Nagisa!"),
        ("Reimu Hakurei", "Hi Reimu Hakurei!"),
    ];

    let hello_addr = format!("http://{}/hello", addr);
    for (name, expected_response) in tests {
        let response = client
            .post(&hello_addr)
            .header("Content-Type", "application/json")
            .form(&HelloRequest { name: name.into() })
            .send()
            .await
            .expect("Failed to send test request")
            .text()
            .await
            .expect("Failed to parse response");
        assert_eq!(expected_response, response);
    }
}

#[tokio::test]
async fn error_for_neither_json_or_form() {
    let addr = test_setup().await;
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("http://{}/hello", addr))
        .body("")
        .send()
        .await
        .expect("Error sending request");
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn bye_returns_string_for_both() {
    let addr = test_setup().await;
    let req = ByeRequest {
        name: "Mark".into(),
    };
    let client = reqwest::Client::new();
    let form_response = client
        .post(&format!("http://{}/bye", addr))
        .form(&req)
        .send()
        .await
        .expect("Error sending request");
    let json_response = client
        .post(&format!("http://{}/bye", addr))
        .json(&req)
        .send()
        .await
        .expect("Error sending request");

    assert_eq!(
        json_response.text().await.unwrap(),
        form_response.text().await.unwrap()
    );
}
