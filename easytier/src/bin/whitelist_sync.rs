use std::path::PathBuf;
use std::time::Duration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let url = std::env::var("WHITELIST_SYNC_URL")
        .expect("WHITELIST_SYNC_URL environment variable not set");
    let file = PathBuf::from(
        std::env::var("IP_WHITELIST_FILE")
            .unwrap_or_else(|_| "/tmp/ip_whitelist.json".into()),
    );
    let interval_secs: u64 = std::env::var("WHITELIST_SYNC_INTERVAL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);

    tracing::info!(
        "whitelist-sync-daemon starting: url={}, file={:?}, interval={}s",
        url,
        file,
        interval_secs
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client");

    let mut shutdown = Box::pin(tokio::signal::ctrl_c());

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                tracing::info!("Received shutdown signal, exiting");
                break;
            }
            result = sync_once(&client, &url, &file) => {
                if let Err(e) = result {
                    tracing::error!("Sync error: {}", e);
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}

async fn sync_once(
    client: &reqwest::Client,
    url: &str,
    file: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    match client.get(url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                tracing::warn!(
                    "Admin returned non-success status: {}, keeping last whitelist",
                    resp.status()
                );
                return Ok(());
            }
            let body = resp.bytes().await?;
            match serde_json::from_slice::<serde_json::Value>(&body) {
                Ok(json) => {
                    if !json.is_array() {
                        tracing::warn!(
                            "Admin response is not an array, keeping last whitelist"
                        );
                        return Ok(());
                    }
                    let tmp = file.with_extension("json.tmp");
                    let content = serde_json::to_string_pretty(&json)?;
                    tokio::fs::write(&tmp, content).await?;
                    tokio::fs::rename(&tmp, file).await?;
                    tracing::info!(
                        "Whitelist synced: {} entries",
                        json.as_array().map(|a| a.len()).unwrap_or(0)
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse admin JSON response: {}, keeping last whitelist",
                        e
                    );
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                "Failed to reach admin at {}: {}, keeping last whitelist",
                url,
                e
            );
        }
    }
    Ok(())
}
