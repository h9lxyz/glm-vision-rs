use serde::{Deserialize, Serialize};

/// Known API providers for GLM-4V.
///
/// Use `Provider::base_url()` to get the corresponding endpoint,
/// or `VisionConfig::with_provider()` to configure the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    /// Zhipu Open Platform — `https://open.bigmodel.cn/api/paas/v4/`
    Zhipu,
    /// Z.AI API — `https://api.z.ai/api/paas/v4/`
    Zai,
    /// Z.AI Coding Plan — `https://api.z.ai/api/coding/paas/v4/`
    ZaiCoding,
}

impl Provider {
    /// Returns the base URL for this provider.
    pub fn base_url(&self) -> &'static str {
        match self {
            Provider::Zhipu => "https://open.bigmodel.cn/api/paas/v4/",
            Provider::Zai => "https://api.z.ai/api/paas/v4/",
            Provider::ZaiCoding => "https://api.z.ai/api/coding/paas/v4/",
        }
    }
}

/// Configuration for the GLM-4V vision client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionConfig {
    /// Vision model name (default: "glm-4.6v")
    #[serde(default = "default_model")]
    pub model: String,

    /// API base URL. Set via `with_provider()` or `with_base_url()`.
    #[serde(default)]
    pub base_url: String,

    /// API key (required)
    pub api_key: String,

    /// Sampling temperature (default: 0.8)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Top-p sampling (default: 0.6)
    #[serde(default = "default_top_p")]
    pub top_p: f32,

    /// Maximum tokens in response (default: 32768)
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Request timeout in seconds (default: 300)
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    /// Maximum image file size in MB (default: 5)
    #[serde(default = "default_max_image_size_mb")]
    pub max_image_size_mb: u64,

    /// Maximum video file size in MB (default: 8)
    #[serde(default = "default_max_video_size_mb")]
    pub max_video_size_mb: u64,

    /// Enable thinking/reasoning mode (default: true)
    #[serde(default = "default_thinking_enabled")]
    pub thinking_enabled: bool,
}

impl VisionConfig {
    /// Create a new VisionConfig with the given API key and defaults.
    ///
    /// You must set a provider via `with_provider()` or a custom URL via `with_base_url()`
    /// before making API calls.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: default_model(),
            base_url: String::new(),
            temperature: default_temperature(),
            top_p: default_top_p(),
            max_tokens: default_max_tokens(),
            timeout_secs: default_timeout_secs(),
            max_image_size_mb: default_max_image_size_mb(),
            max_video_size_mb: default_max_video_size_mb(),
            thinking_enabled: default_thinking_enabled(),
        }
    }

    /// Set the provider (sets the base URL automatically).
    pub fn with_provider(mut self, provider: Provider) -> Self {
        self.base_url = provider.base_url().to_string();
        self
    }

    /// Set a custom base URL (overrides provider).
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the model name.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set thinking mode on or off.
    pub fn with_thinking(mut self, enabled: bool) -> Self {
        self.thinking_enabled = enabled;
        self
    }

    /// Returns the full chat completions URL.
    pub fn completions_url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        format!("{}/chat/completions", base)
    }
}

fn default_model() -> String {
    "glm-4.6v".to_string()
}

fn default_temperature() -> f32 {
    0.8
}

fn default_top_p() -> f32 {
    0.6
}

fn default_max_tokens() -> u32 {
    32768
}

fn default_timeout_secs() -> u64 {
    300
}

fn default_max_image_size_mb() -> u64 {
    5
}

fn default_max_video_size_mb() -> u64 {
    8
}

fn default_thinking_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config_defaults() {
        let config = VisionConfig::new("test-key");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.model, "glm-4.6v");
        assert!(config.base_url.is_empty());
        assert_eq!(config.temperature, 0.8);
        assert_eq!(config.top_p, 0.6);
        assert_eq!(config.max_tokens, 32768);
        assert_eq!(config.timeout_secs, 300);
        assert!(config.thinking_enabled);
    }

    #[test]
    fn test_provider_urls() {
        assert_eq!(
            Provider::Zhipu.base_url(),
            "https://open.bigmodel.cn/api/paas/v4/"
        );
        assert_eq!(Provider::Zai.base_url(), "https://api.z.ai/api/paas/v4/");
        assert_eq!(
            Provider::ZaiCoding.base_url(),
            "https://api.z.ai/api/coding/paas/v4/"
        );
    }

    #[test]
    fn test_with_provider() {
        let config = VisionConfig::new("key").with_provider(Provider::ZaiCoding);
        assert_eq!(config.base_url, "https://api.z.ai/api/coding/paas/v4/");
    }

    #[test]
    fn test_base_url_overrides_provider() {
        let config = VisionConfig::new("key")
            .with_provider(Provider::Zhipu)
            .with_base_url("https://custom.example.com/v1/");
        assert_eq!(config.base_url, "https://custom.example.com/v1/");
    }

    #[test]
    fn test_builder_methods() {
        let config = VisionConfig::new("key")
            .with_provider(Provider::Zai)
            .with_model("glm-4.7v")
            .with_temperature(0.5)
            .with_thinking(false);
        assert_eq!(config.base_url, "https://api.z.ai/api/paas/v4/");
        assert_eq!(config.model, "glm-4.7v");
        assert_eq!(config.temperature, 0.5);
        assert!(!config.thinking_enabled);
    }

    #[test]
    fn test_completions_url() {
        let config = VisionConfig::new("key").with_provider(Provider::Zhipu);
        assert!(config.completions_url().ends_with("/chat/completions"));

        let config2 = VisionConfig::new("key").with_base_url("https://api.z.ai/api/paas/v4");
        assert_eq!(
            config2.completions_url(),
            "https://api.z.ai/api/paas/v4/chat/completions"
        );
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = VisionConfig::new("my-key").with_provider(Provider::Zhipu);
        let json = serde_json::to_string(&config).unwrap();
        let parsed: VisionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.api_key, "my-key");
        assert_eq!(parsed.model, "glm-4.6v");
        assert_eq!(parsed.base_url, "https://open.bigmodel.cn/api/paas/v4/");
    }

    #[test]
    fn test_provider_serde() {
        let json = serde_json::to_string(&Provider::ZaiCoding).unwrap();
        assert_eq!(json, "\"zai-coding\"");
        let parsed: Provider = serde_json::from_str("\"zhipu\"").unwrap();
        assert_eq!(parsed, Provider::Zhipu);
    }
}
