//! Storage tools - IPFS and decentralized storage operations
//!
//! These tools provide IPFS upload/download capabilities.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use super::super::dispatcher::{DispatchError, ToolHandler, ToolOutput};
use super::super::intent::IntentParams;

/// Default IPFS gateway
const DEFAULT_GATEWAY: &str = "https://ipfs.io/ipfs/";

/// Alternative gateways for fallback
const FALLBACK_GATEWAYS: &[&str] = &[
    "https://gateway.pinata.cloud/ipfs/",
    "https://cloudflare-ipfs.com/ipfs/",
    "https://dweb.link/ipfs/",
];

/// Upload to IPFS tool
pub struct UploadIPFSTool {
    api_endpoint: String,
}

impl UploadIPFSTool {
    pub fn new() -> Self {
        Self {
            api_endpoint: "http://localhost:5001/api/v0".to_string(),
        }
    }

    pub fn with_endpoint(endpoint: &str) -> Self {
        Self {
            api_endpoint: endpoint.to_string(),
        }
    }
}

impl Default for UploadIPFSTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolHandler for UploadIPFSTool {
    fn name(&self) -> &str {
        "upload_ipfs"
    }

    fn description(&self) -> &str {
        "Upload content (file or data) to IPFS and receive a CID"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let api_endpoint = self.api_endpoint.clone();
        let file_path = params.prompt.clone(); // Path to file
        let content_data = params.contract_data.clone(); // Or raw data
        Box::pin(async move {
            // Determine what to upload
            let (content, content_type, size) = if let Some(path) = file_path {
                // Upload from file path
                let path = PathBuf::from(&path);
                if !path.exists() {
                    return Ok(ToolOutput {
                        tool: "upload_ipfs".to_string(),
                        success: false,
                        message: format!("File not found: {}", path.display()),
                        data: None,
                    });
                }

                match tokio::fs::read(&path).await {
                    Ok(bytes) => {
                        let filename = path
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_else(|| "file".to_string());
                        (bytes, filename, path.metadata().map(|m| m.len()).ok())
                    }
                    Err(e) => {
                        return Ok(ToolOutput {
                            tool: "upload_ipfs".to_string(),
                            success: false,
                            message: format!("Failed to read file: {}", e),
                            data: None,
                        });
                    }
                }
            } else if let Some(data) = content_data {
                // Upload raw data
                let bytes = if data.starts_with("0x") {
                    hex::decode(&data[2..]).unwrap_or_else(|_| data.into_bytes())
                } else {
                    data.into_bytes()
                };
                let len = bytes.len();
                (bytes, "data".to_string(), Some(len as u64))
            } else {
                return Err(DispatchError::InvalidParams(
                    "Either file path or content data required".to_string(),
                ));
            };

            // Try to upload via local IPFS node
            let client = reqwest::Client::new();
            let form = reqwest::multipart::Form::new().part(
                "file",
                reqwest::multipart::Part::bytes(content.clone())
                    .file_name(content_type.clone()),
            );

            match client
                .post(format!("{}/add", api_endpoint))
                .multipart(form)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(json) = response.json::<serde_json::Value>().await {
                            let cid = json
                                .get("Hash")
                                .and_then(|h| h.as_str())
                                .unwrap_or("unknown");

                            return Ok(ToolOutput {
                                tool: "upload_ipfs".to_string(),
                                success: true,
                                message: format!(
                                    "Uploaded to IPFS! CID: {}",
                                    cid
                                ),
                                data: Some(serde_json::json!({
                                    "cid": cid,
                                    "gateway_url": format!("{}{}", DEFAULT_GATEWAY, cid),
                                    "size_bytes": size,
                                    "content_name": content_type
                                })),
                            });
                        }
                    }
                }
                Err(e) => {
                    // IPFS node not available - return proper error, no mock fallback
                    return Ok(ToolOutput {
                        tool: "upload_ipfs".to_string(),
                        success: false,
                        message: format!(
                            "IPFS upload failed: local node not available at {}",
                            api_endpoint
                        ),
                        data: Some(serde_json::json!({
                            "error": format!("{}", e),
                            "size_bytes": size,
                            "content_name": content_type,
                            "suggestion": "Please start a local IPFS node (ipfs daemon) or configure a pinning service. \
                                           Install IPFS: https://docs.ipfs.tech/install/"
                        })),
                    });
                }
            }

            // Should not reach here - all paths return above
            Ok(ToolOutput {
                tool: "upload_ipfs".to_string(),
                success: false,
                message: "IPFS upload failed: no response from IPFS node".to_string(),
                data: Some(serde_json::json!({
                    "suggestion": "Ensure IPFS daemon is running and accessible"
                })),
            })
        })
    }
}

/// Get IPFS content tool
pub struct GetIPFSTool;

impl GetIPFSTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GetIPFSTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolHandler for GetIPFSTool {
    fn name(&self) -> &str {
        "get_ipfs"
    }

    fn description(&self) -> &str {
        "Retrieve content from IPFS by CID"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let cid = params.prompt.clone(); // CID to retrieve
        Box::pin(async move {
            let cid = cid.ok_or_else(|| {
                DispatchError::InvalidParams("CID required".to_string())
            })?;

            // Validate CID format (basic check)
            if !cid.starts_with("Qm") && !cid.starts_with("bafy") {
                return Ok(ToolOutput {
                    tool: "get_ipfs".to_string(),
                    success: false,
                    message: format!("Invalid CID format: {}", cid),
                    data: None,
                });
            }

            // Try to fetch from gateways
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| DispatchError::Internal(e.to_string()))?;

            let mut last_error = String::new();

            for gateway in std::iter::once(DEFAULT_GATEWAY).chain(FALLBACK_GATEWAYS.iter().copied())
            {
                let url = format!("{}{}", gateway, cid);
                match client.get(&url).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            let content_type = response
                                .headers()
                                .get("content-type")
                                .and_then(|h| h.to_str().ok())
                                .unwrap_or("application/octet-stream")
                                .to_string();

                            let content_length = response.content_length();

                            // For text content, read and return
                            if content_type.starts_with("text/")
                                || content_type.contains("json")
                                || content_type.contains("javascript")
                            {
                                if let Ok(text) = response.text().await {
                                    let preview = if text.len() > 500 {
                                        format!("{}... (truncated)", &text[..500])
                                    } else {
                                        text.clone()
                                    };

                                    return Ok(ToolOutput {
                                        tool: "get_ipfs".to_string(),
                                        success: true,
                                        message: format!(
                                            "Retrieved {} ({} bytes)",
                                            cid,
                                            text.len()
                                        ),
                                        data: Some(serde_json::json!({
                                            "cid": cid,
                                            "content_type": content_type,
                                            "size_bytes": text.len(),
                                            "gateway": gateway,
                                            "content": preview
                                        })),
                                    });
                                }
                            }

                            // For binary content, just return metadata
                            return Ok(ToolOutput {
                                tool: "get_ipfs".to_string(),
                                success: true,
                                message: format!(
                                    "Found {} ({}, {} bytes)",
                                    cid,
                                    content_type,
                                    content_length.unwrap_or(0)
                                ),
                                data: Some(serde_json::json!({
                                    "cid": cid,
                                    "content_type": content_type,
                                    "size_bytes": content_length,
                                    "gateway": gateway,
                                    "gateway_url": url
                                })),
                            });
                        }
                    }
                    Err(e) => {
                        last_error = e.to_string();
                        continue;
                    }
                }
            }

            let tried: Vec<&str> = std::iter::once(DEFAULT_GATEWAY)
                .chain(FALLBACK_GATEWAYS.iter().copied())
                .collect();

            Ok(ToolOutput {
                tool: "get_ipfs".to_string(),
                success: false,
                message: format!(
                    "Failed to retrieve CID '{}' from any gateway: {}",
                    cid, last_error
                ),
                data: Some(serde_json::json!({
                    "cid": cid,
                    "tried_gateways": tried
                })),
            })
        })
    }
}

/// Pin IPFS content tool
pub struct PinIPFSTool {
    api_endpoint: String,
}

impl PinIPFSTool {
    pub fn new() -> Self {
        Self {
            api_endpoint: "http://localhost:5001/api/v0".to_string(),
        }
    }
}

impl Default for PinIPFSTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolHandler for PinIPFSTool {
    fn name(&self) -> &str {
        "pin_ipfs"
    }

    fn description(&self) -> &str {
        "Pin IPFS content to ensure it remains available"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let api_endpoint = self.api_endpoint.clone();
        let cid = params.prompt.clone(); // CID to pin
        Box::pin(async move {
            let cid = cid.ok_or_else(|| {
                DispatchError::InvalidParams("CID required".to_string())
            })?;

            // Try to pin via local IPFS node
            let client = reqwest::Client::new();
            match client
                .post(format!("{}/pin/add?arg={}", api_endpoint, cid))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(ToolOutput {
                            tool: "pin_ipfs".to_string(),
                            success: true,
                            message: format!("Successfully pinned CID: {}", cid),
                            data: Some(serde_json::json!({
                                "cid": cid,
                                "pinned": true
                            })),
                        });
                    }
                }
                Err(_) => {}
            }

            // Fallback message
            Ok(ToolOutput {
                tool: "pin_ipfs".to_string(),
                success: false,
                message: format!(
                    "Could not pin CID '{}'. Local IPFS node not available. Use a pinning service.",
                    cid
                ),
                data: Some(serde_json::json!({
                    "cid": cid,
                    "pinned": false,
                    "suggestion": "Use Pinata, Infura, or Web3.Storage for pinning"
                })),
            })
        })
    }
}

/// Simple SHA3-256 hash for mock CID generation
fn sha256_hash(data: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Sha3_256};
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_ipfs_tool_name() {
        let tool = UploadIPFSTool::new();
        assert_eq!(tool.name(), "upload_ipfs");
    }

    #[test]
    fn test_upload_ipfs_tool_description() {
        let tool = UploadIPFSTool::new();
        assert!(tool.description().contains("Upload"));
        assert!(tool.description().contains("IPFS"));
    }

    #[test]
    fn test_upload_ipfs_default() {
        let tool = UploadIPFSTool::default();
        assert_eq!(tool.api_endpoint, "http://localhost:5001/api/v0");
    }

    #[test]
    fn test_upload_ipfs_with_endpoint() {
        let tool = UploadIPFSTool::with_endpoint("http://custom:5001/api/v0");
        assert_eq!(tool.api_endpoint, "http://custom:5001/api/v0");
    }

    #[test]
    fn test_get_ipfs_tool_name() {
        let tool = GetIPFSTool::new();
        assert_eq!(tool.name(), "get_ipfs");
    }

    #[test]
    fn test_get_ipfs_tool_description() {
        let tool = GetIPFSTool::new();
        assert!(tool.description().contains("Retrieve"));
        assert!(tool.description().contains("CID"));
    }

    #[test]
    fn test_get_ipfs_default() {
        let _tool = GetIPFSTool::default();
        // Just verify default creation works
    }

    #[test]
    fn test_pin_ipfs_tool_name() {
        let tool = PinIPFSTool::new();
        assert_eq!(tool.name(), "pin_ipfs");
    }

    #[test]
    fn test_pin_ipfs_tool_description() {
        let tool = PinIPFSTool::new();
        assert!(tool.description().contains("Pin"));
    }

    #[test]
    fn test_pin_ipfs_default() {
        let tool = PinIPFSTool::default();
        assert_eq!(tool.api_endpoint, "http://localhost:5001/api/v0");
    }

    #[test]
    fn test_default_gateway_constant() {
        assert!(DEFAULT_GATEWAY.starts_with("https://"));
        assert!(DEFAULT_GATEWAY.contains("ipfs"));
    }

    #[test]
    fn test_fallback_gateways_constant() {
        assert!(FALLBACK_GATEWAYS.len() >= 3);
        for gateway in FALLBACK_GATEWAYS {
            assert!(gateway.starts_with("https://"));
            assert!(gateway.contains("ipfs"));
        }
    }

    #[test]
    fn test_sha256_hash() {
        let data = b"test data";
        let hash = sha256_hash(data);
        assert_eq!(hash.len(), 32);

        // Same input should produce same hash
        let hash2 = sha256_hash(data);
        assert_eq!(hash, hash2);

        // Different input should produce different hash
        let hash3 = sha256_hash(b"different data");
        assert_ne!(hash, hash3);
    }

    #[tokio::test]
    async fn test_upload_ipfs_missing_params() {
        let tool = UploadIPFSTool::new();
        let params = IntentParams::default();

        let result = tool.execute(&params).await;
        assert!(result.is_err());

        if let Err(DispatchError::InvalidParams(msg)) = result {
            assert!(msg.contains("required"));
        } else {
            panic!("Expected InvalidParams error");
        }
    }

    #[tokio::test]
    async fn test_upload_ipfs_file_not_found() {
        let tool = UploadIPFSTool::new();
        let mut params = IntentParams::default();
        params.prompt = Some("/nonexistent/path/to/file.txt".to_string());

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.success);
        assert!(output.message.contains("not found"));
    }

    #[tokio::test]
    async fn test_get_ipfs_missing_cid() {
        let tool = GetIPFSTool::new();
        let params = IntentParams::default();

        let result = tool.execute(&params).await;
        assert!(result.is_err());

        if let Err(DispatchError::InvalidParams(msg)) = result {
            assert!(msg.contains("CID required"));
        } else {
            panic!("Expected InvalidParams error");
        }
    }

    #[tokio::test]
    async fn test_get_ipfs_invalid_cid_format() {
        let tool = GetIPFSTool::new();
        let mut params = IntentParams::default();
        params.prompt = Some("invalid_cid".to_string());

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.success);
        assert!(output.message.contains("Invalid CID format"));
    }

    #[tokio::test]
    async fn test_pin_ipfs_missing_cid() {
        let tool = PinIPFSTool::new();
        let params = IntentParams::default();

        let result = tool.execute(&params).await;
        assert!(result.is_err());

        if let Err(DispatchError::InvalidParams(msg)) = result {
            assert!(msg.contains("CID required"));
        } else {
            panic!("Expected InvalidParams error");
        }
    }

    #[tokio::test]
    async fn test_pin_ipfs_no_node_available() {
        let tool = PinIPFSTool::new();
        let mut params = IntentParams::default();
        params.prompt = Some("QmTestCidHash123456789".to_string());

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        // Should fail gracefully when IPFS node is not available
        assert!(!output.success);
        assert!(output.message.contains("not available") || output.message.contains("Could not pin"));
    }
}
