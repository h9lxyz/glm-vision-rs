use anyhow::{bail, Context, Result};
use std::path::Path;

use crate::config::VisionConfig;
use crate::types::*;

/// HTTP client for the GLM-4V vision completions API.
pub struct VisionClient {
    http: reqwest::Client,
    config: VisionConfig,
}

impl VisionClient {
    /// Create a new VisionClient with the given configuration.
    pub fn new(config: VisionConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_default();
        Self { http, config }
    }

    /// Returns a reference to the underlying config.
    pub fn config(&self) -> &VisionConfig {
        &self.config
    }

    /// Process an image source (local file path or URL) into a `ContentPart`.
    ///
    /// - URLs are passed through directly.
    /// - Local files are validated, base64-encoded, and wrapped in a data URL.
    pub fn process_image(&self, source: &str) -> Result<ContentPart> {
        if is_url(source) {
            return Ok(ContentPart::ImageUrl {
                image_url: UrlHolder {
                    url: source.to_string(),
                },
            });
        }

        let path = Path::new(source);
        if !path.exists() {
            bail!("Image file not found: {}", source);
        }

        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to read metadata: {}", source))?;
        let size_mb = metadata.len() / (1024 * 1024);
        if size_mb > self.config.max_image_size_mb {
            bail!(
                "Image file too large: {}MB (max {}MB)",
                size_mb,
                self.config.max_image_size_mb
            );
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let mime = image_mime_type(ext)
            .with_context(|| format!("Unsupported image format: .{}", ext))?;

        let data = std::fs::read(path)
            .with_context(|| format!("Failed to read image: {}", source))?;
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
        let data_url = format!("data:{};base64,{}", mime, b64);

        Ok(ContentPart::ImageUrl {
            image_url: UrlHolder { url: data_url },
        })
    }

    /// Process a video source (local file path or URL) into a `ContentPart`.
    ///
    /// - URLs are passed through directly.
    /// - Local files are validated, base64-encoded, and wrapped in a data URL.
    pub fn process_video(&self, source: &str) -> Result<ContentPart> {
        if is_url(source) {
            return Ok(ContentPart::VideoUrl {
                video_url: UrlHolder {
                    url: source.to_string(),
                },
            });
        }

        let path = Path::new(source);
        if !path.exists() {
            bail!("Video file not found: {}", source);
        }

        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to read metadata: {}", source))?;
        let size_mb = metadata.len() / (1024 * 1024);
        if size_mb > self.config.max_video_size_mb {
            bail!(
                "Video file too large: {}MB (max {}MB)",
                size_mb,
                self.config.max_video_size_mb
            );
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let mime = video_mime_type(ext)
            .with_context(|| format!("Unsupported video format: .{}", ext))?;

        let data = std::fs::read(path)
            .with_context(|| format!("Failed to read video: {}", source))?;
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
        let data_url = format!("data:{};base64,{}", mime, b64);

        Ok(ContentPart::VideoUrl {
            video_url: UrlHolder { url: data_url },
        })
    }

    /// Send a vision completion request and return the raw JSON response body.
    pub async fn completion_raw(
        &self,
        system_prompt: &str,
        content_parts: Vec<ContentPart>,
        user_prompt: &str,
    ) -> Result<String> {
        let mut user_content = content_parts;
        user_content.push(ContentPart::Text {
            text: user_prompt.to_string(),
        });

        let thinking = if self.config.thinking_enabled {
            Some(ThinkingConfig {
                kind: "enabled".to_string(),
            })
        } else {
            None
        };

        let request = VisionChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                VisionMessage {
                    role: "system".to_string(),
                    content: VisionContent::Text(system_prompt.to_string()),
                },
                VisionMessage {
                    role: "user".to_string(),
                    content: VisionContent::Parts(user_content),
                },
            ],
            thinking,
            stream: false,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            max_tokens: self.config.max_tokens,
        };

        let url = self.config.completions_url();
        log::info!("Requesting vision completion: model={}", self.config.model);

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .header("X-Title", "4.5V MCP Local")
            .header("Accept-Language", "en-US,en")
            .json(&request)
            .send()
            .await
            .context("Failed to send vision API request")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            bail!("Vision API error HTTP {}: {}", status, body);
        }

        response
            .text()
            .await
            .context("Failed to read vision API response body")
    }

    /// Send a vision completion request and return the extracted text content.
    pub async fn completion(
        &self,
        system_prompt: &str,
        content_parts: Vec<ContentPart>,
        user_prompt: &str,
    ) -> Result<String> {
        let raw = self
            .completion_raw(system_prompt, content_parts, user_prompt)
            .await?;

        let chat_response: VisionChatResponse = serde_json::from_str(&raw)
            .context("Failed to parse vision API response")?;

        chat_response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .context("Vision API response missing content")
    }

    /// Same as `completion` but with exponential-backoff retry for transient errors
    /// (HTTP 429 and 5xx).
    pub async fn completion_with_retry(
        &self,
        system_prompt: &str,
        content_parts: Vec<ContentPart>,
        user_prompt: &str,
        max_retries: u32,
    ) -> Result<String> {
        let mut last_err = None;

        for attempt in 0..=max_retries {
            match self
                .completion(system_prompt, content_parts.clone(), user_prompt)
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let err_str = e.to_string();
                    let is_retryable = err_str.contains("HTTP 429")
                        || err_str.contains("HTTP 5");

                    if !is_retryable || attempt == max_retries {
                        return Err(e);
                    }

                    let wait = std::time::Duration::from_millis(1000 * 2u64.pow(attempt));
                    log::warn!(
                        "Vision API request failed (attempt {}/{}), retrying in {:?}: {}",
                        attempt + 1,
                        max_retries + 1,
                        wait,
                        err_str
                    );
                    tokio::time::sleep(wait).await;
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("Max retries exceeded")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_image_url_passthrough() {
        let client = VisionClient::new(VisionConfig::new("test-key"));
        let part = client
            .process_image("https://example.com/img.png")
            .unwrap();
        match part {
            ContentPart::ImageUrl { image_url } => {
                assert_eq!(image_url.url, "https://example.com/img.png");
            }
            _ => panic!("expected ImageUrl"),
        }
    }

    #[test]
    fn test_process_video_url_passthrough() {
        let client = VisionClient::new(VisionConfig::new("test-key"));
        let part = client
            .process_video("https://example.com/vid.mp4")
            .unwrap();
        match part {
            ContentPart::VideoUrl { video_url } => {
                assert_eq!(video_url.url, "https://example.com/vid.mp4");
            }
            _ => panic!("expected VideoUrl"),
        }
    }

    #[test]
    fn test_process_image_file_not_found() {
        let client = VisionClient::new(VisionConfig::new("test-key"));
        let result = client.process_image("/nonexistent/image.png");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_process_video_file_not_found() {
        let client = VisionClient::new(VisionConfig::new("test-key"));
        let result = client.process_video("/nonexistent/video.mp4");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_process_image_local_file() {
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("test.png");
        // Write a minimal valid byte sequence (not a real PNG, but enough for encoding)
        std::fs::write(&img_path, b"fake-png-data").unwrap();

        let client = VisionClient::new(VisionConfig::new("test-key"));
        let part = client
            .process_image(img_path.to_str().unwrap())
            .unwrap();
        match part {
            ContentPart::ImageUrl { image_url } => {
                assert!(image_url.url.starts_with("data:image/png;base64,"));
            }
            _ => panic!("expected ImageUrl with data URL"),
        }
    }

    #[test]
    fn test_process_video_local_file() {
        let dir = tempfile::tempdir().unwrap();
        let vid_path = dir.path().join("test.mp4");
        std::fs::write(&vid_path, b"fake-mp4-data").unwrap();

        let client = VisionClient::new(VisionConfig::new("test-key"));
        let part = client
            .process_video(vid_path.to_str().unwrap())
            .unwrap();
        match part {
            ContentPart::VideoUrl { video_url } => {
                assert!(video_url.url.starts_with("data:video/mp4;base64,"));
            }
            _ => panic!("expected VideoUrl with data URL"),
        }
    }

    #[test]
    fn test_process_image_unsupported_format() {
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("test.gif");
        std::fs::write(&img_path, b"fake-gif").unwrap();

        let client = VisionClient::new(VisionConfig::new("test-key"));
        let result = client.process_image(img_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported"));
    }
}
