use futures::StreamExt;
use serde::de::DeserializeOwned;
use tokio::sync::mpsc;
use tracing::warn;

/// Parse an SSE stream from a reqwest response, sending parsed events through a channel.
/// Returns when the stream ends or the channel is closed.
pub async fn parse_sse_stream<T: DeserializeOwned + Send + 'static>(
    response: reqwest::Response,
    tx: mpsc::Sender<T>,
) -> Result<(), String> {
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                let chunk_str = String::from_utf8_lossy(&chunk);
                buffer.push_str(&chunk_str);

                while let Some(pos) = buffer.find("\n\n") {
                    let event = buffer[..pos].to_string();
                    buffer = buffer[pos + 2..].to_string();

                    for line in event.lines() {
                        if let Some(data) = line.strip_prefix("data:") {
                            let data = data.trim();
                            if data.is_empty() {
                                continue;
                            }
                            match serde_json::from_str::<T>(data) {
                                Ok(parsed) => {
                                    if tx.send(parsed).await.is_err() {
                                        return Ok(());
                                    }
                                }
                                Err(e) => {
                                    warn!(error = %e, "Failed to parse SSE event");
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                return Err(format!("SSE stream error: {}", e));
            }
        }
    }
    Ok(())
}
