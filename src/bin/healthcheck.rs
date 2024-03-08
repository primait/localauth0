use localauth0::config::Config;

async fn is_endpoint_healthy(client: &reqwest::Client, endpoint: String) -> bool {
    let status = client.get(&endpoint).send().await.map(|r| r.error_for_status());

    match status {
        Ok(Ok(_)) => {
            println!("{endpoint} OK");
            true
        }
        _ => {
            println!("{endpoint} ERROR {status:?}");
            false
        }
    }
}

fn main() -> Result<(), ()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let config = Config::load_or_default();

            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap();

            let endpoint_http = format!("http://127.0.0.1:{}/healthcheck", config.http().port());
            let endpoint_https = format!("https://127.0.0.1:{}/healthcheck", config.https().port());

            let http_healthy = is_endpoint_healthy(&client, endpoint_http).await;
            let https_healthy = is_endpoint_healthy(&client, endpoint_https).await;

            if http_healthy && https_healthy {
                Ok(())
            } else {
                Err(())
            }
        })
}
