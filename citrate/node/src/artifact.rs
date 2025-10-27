use async_trait::async_trait;
use citrate_execution::executor::ArtifactService;
use citrate_execution::ExecutionError;
use tokio::time::{sleep, Duration};

/// Simple IPFS HTTP client-backed artifact service
pub struct NodeArtifactService {
    client: reqwest::Client,
    apis: Vec<String>,
}

impl NodeArtifactService {
    pub fn new(api_base: Option<String>) -> Self {
        // Prefer multi-provider list from env, fallback to single base
        let apis = if let Ok(list) = std::env::var("CITRATE_IPFS_PROVIDERS") {
            list.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            vec![api_base.unwrap_or_else(|| "http://127.0.0.1:5001".to_string())]
        };
        Self {
            client: reqwest::Client::new(),
            apis,
        }
    }

    pub fn new_with_providers(providers: Vec<String>) -> Self {
        let apis = if providers.is_empty() {
            vec!["http://127.0.0.1:5001".to_string()]
        } else {
            providers
        };
        Self {
            client: reqwest::Client::new(),
            apis,
        }
    }
}

#[async_trait]
impl ArtifactService for NodeArtifactService {
    async fn pin(&self, cid: &str, replicas: usize) -> Result<(), ExecutionError> {
        let needed = replicas.max(1);
        let mut successes = 0usize;
        let mut last_err: Option<String> = None;
        for base in &self.apis {
            // Up to 3 attempts with exponential backoff
            let mut attempt = 0;
            loop {
                let url = format!("{}/api/v0/pin/add?arg={}", base, cid);
                match self.client.post(&url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        successes += 1;
                        break;
                    }
                    Ok(resp) => {
                        last_err = Some(format!("{}: status {}", base, resp.status()));
                    }
                    Err(e) => {
                        last_err = Some(format!("{}: {}", base, e));
                    }
                }
                attempt += 1;
                if attempt >= 3 {
                    break;
                }
                let backoff = 2u64.pow(attempt) * 100; // 100ms, 200ms, 400ms
                sleep(Duration::from_millis(backoff)).await;
            }
            if successes >= needed {
                return Ok(());
            }
        }
        Err(ExecutionError::Reverted(
            last_err.unwrap_or_else(|| "pin failed".into()),
        ))
    }

    async fn status(&self, cid: &str) -> Result<String, ExecutionError> {
        // Return JSON array of per-provider statuses
        let mut arr = Vec::new();
        for base in &self.apis {
            let url = format!("{}/api/v0/pin/ls?arg={}", base, cid);
            let status = match self.client.post(&url).send().await {
                Ok(resp) if resp.status().is_success() => match resp.text().await {
                    Ok(body) => {
                        if body.contains(cid) {
                            "pinned"
                        } else {
                            "unpinned"
                        }
                    }
                    Err(_) => "unknown",
                },
                _ => "unknown",
            };
            arr.push(serde_json::json!({ "provider": base, "status": status }));
        }
        Ok(serde_json::to_string(&arr).unwrap_or_else(|_| "[]".into()))
    }

    async fn add(&self, data: &[u8]) -> Result<String, ExecutionError> {
        // Add to the first provider
        let base = self
            .apis
            .first()
            .cloned()
            .unwrap_or_else(|| "http://127.0.0.1:5001".to_string());
        let url = format!("{}/api/v0/add?pin=true", base);
        let part = reqwest::multipart::Part::bytes(data.to_vec()).file_name("artifact.bin");
        let form = reqwest::multipart::Form::new().part("file", part);
        let resp = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| ExecutionError::Reverted(format!("ipfs add error: {}", e)))?;
        if !resp.status().is_success() {
            return Err(ExecutionError::Reverted(format!(
                "ipfs add status: {}",
                resp.status()
            )));
        }
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ExecutionError::Reverted(format!("ipfs add parse error: {}", e)))?;
        let cid = json["Hash"].as_str().unwrap_or("").to_string();
        if cid.is_empty() {
            return Err(ExecutionError::Reverted(
                "ipfs add returned empty cid".into(),
            ));
        }
        Ok(cid)
    }
}
