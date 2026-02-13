use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct VisionChatRequest {
    pub model: String,
    pub messages: Vec<VisionMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    pub stream: bool,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct VisionMessage {
    pub role: String,
    pub content: VisionContent,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum VisionContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: UrlHolder },
    #[serde(rename = "video_url")]
    VideoUrl { video_url: UrlHolder },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlHolder {
    pub url: String,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct VisionChatResponse {
    pub choices: Vec<VisionChoice>,
}

#[derive(Debug, Deserialize)]
pub struct VisionChoice {
    pub message: VisionChoiceMessage,
}

#[derive(Debug, Deserialize)]
pub struct VisionChoiceMessage {
    pub content: Option<String>,
}

// ---------------------------------------------------------------------------
// Media helpers
// ---------------------------------------------------------------------------

/// Returns the MIME type for an image file extension, or None if unsupported.
pub fn image_mime_type(ext: &str) -> Option<&'static str> {
    match ext.to_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        _ => None,
    }
}

/// Returns the MIME type for a video file extension, or None if unsupported.
pub fn video_mime_type(ext: &str) -> Option<&'static str> {
    match ext.to_lowercase().as_str() {
        "mp4" => Some("video/mp4"),
        "mov" => Some("video/quicktime"),
        "m4v" => Some("video/x-m4v"),
        "avi" => Some("video/x-msvideo"),
        "webm" => Some("video/webm"),
        "wmv" => Some("video/x-ms-wmv"),
        _ => None,
    }
}

/// Returns true if `source` looks like an HTTP(S) URL.
pub fn is_url(source: &str) -> bool {
    source.starts_with("http://") || source.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_part_text_serialization() {
        let part = ContentPart::Text {
            text: "hello".into(),
        };
        let json = serde_json::to_value(&part).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "hello");
    }

    #[test]
    fn test_content_part_image_url_serialization() {
        let part = ContentPart::ImageUrl {
            image_url: UrlHolder {
                url: "https://example.com/img.png".into(),
            },
        };
        let json = serde_json::to_value(&part).unwrap();
        assert_eq!(json["type"], "image_url");
        assert_eq!(json["image_url"]["url"], "https://example.com/img.png");
    }

    #[test]
    fn test_content_part_video_url_serialization() {
        let part = ContentPart::VideoUrl {
            video_url: UrlHolder {
                url: "data:video/mp4;base64,AAAA".into(),
            },
        };
        let json = serde_json::to_value(&part).unwrap();
        assert_eq!(json["type"], "video_url");
        assert_eq!(json["video_url"]["url"], "data:video/mp4;base64,AAAA");
    }

    #[test]
    fn test_content_part_roundtrip() {
        let part = ContentPart::ImageUrl {
            image_url: UrlHolder {
                url: "https://img.test/a.jpg".into(),
            },
        };
        let json = serde_json::to_string(&part).unwrap();
        let parsed: ContentPart = serde_json::from_str(&json).unwrap();
        match parsed {
            ContentPart::ImageUrl { image_url } => {
                assert_eq!(image_url.url, "https://img.test/a.jpg");
            }
            _ => panic!("expected ImageUrl variant"),
        }
    }

    #[test]
    fn test_image_mime_type() {
        assert_eq!(image_mime_type("png"), Some("image/png"));
        assert_eq!(image_mime_type("jpg"), Some("image/jpeg"));
        assert_eq!(image_mime_type("jpeg"), Some("image/jpeg"));
        assert_eq!(image_mime_type("PNG"), Some("image/png"));
        assert_eq!(image_mime_type("gif"), None);
        assert_eq!(image_mime_type("bmp"), None);
    }

    #[test]
    fn test_video_mime_type() {
        assert_eq!(video_mime_type("mp4"), Some("video/mp4"));
        assert_eq!(video_mime_type("mov"), Some("video/quicktime"));
        assert_eq!(video_mime_type("m4v"), Some("video/x-m4v"));
        assert_eq!(video_mime_type("avi"), Some("video/x-msvideo"));
        assert_eq!(video_mime_type("webm"), Some("video/webm"));
        assert_eq!(video_mime_type("wmv"), Some("video/x-ms-wmv"));
        assert_eq!(video_mime_type("flv"), None);
    }

    #[test]
    fn test_is_url() {
        assert!(is_url("https://example.com/image.png"));
        assert!(is_url("http://localhost:8080/video.mp4"));
        assert!(!is_url("/home/user/image.png"));
        assert!(!is_url("./relative/path.jpg"));
        assert!(!is_url("ftp://server/file"));
    }

    #[test]
    fn test_vision_chat_request_serialization() {
        let req = VisionChatRequest {
            model: "glm-4.6v".into(),
            messages: vec![
                VisionMessage {
                    role: "system".into(),
                    content: VisionContent::Text("You are a vision assistant.".into()),
                },
                VisionMessage {
                    role: "user".into(),
                    content: VisionContent::Parts(vec![
                        ContentPart::ImageUrl {
                            image_url: UrlHolder {
                                url: "https://example.com/img.png".into(),
                            },
                        },
                        ContentPart::Text {
                            text: "Describe this image.".into(),
                        },
                    ]),
                },
            ],
            thinking: Some(ThinkingConfig {
                kind: "enabled".into(),
            }),
            stream: false,
            temperature: 0.8,
            top_p: 0.6,
            max_tokens: 32768,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "glm-4.6v");
        assert_eq!(json["messages"][0]["role"], "system");
        assert_eq!(json["messages"][1]["content"][0]["type"], "image_url");
        assert_eq!(json["thinking"]["type"], "enabled");
        assert_eq!(json["stream"], false);
    }
}
