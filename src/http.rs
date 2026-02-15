use std::future::Future;

/// Response from an HTTP request.
#[derive(Debug)]
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response body as text.
    pub body: String,
}

impl HttpResponse {
    /// Returns `true` if the status code is in the 2xx range.
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Trait for making HTTP POST requests.
///
/// Implement this with your preferred HTTP client (e.g. reqwest, ureq, hyper)
/// to plug it into [`VisionClient`](crate::VisionClient).
///
/// # Example (reqwest)
///
/// ```ignore
/// struct ReqwestClient(reqwest::Client);
///
/// impl glm_vision_rs::HttpClient for ReqwestClient {
///     async fn post(
///         &self,
///         url: &str,
///         headers: &[(&str, &str)],
///         body: &[u8],
///     ) -> Result<glm_vision_rs::HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
///         let mut req = self.0.post(url);
///         for &(k, v) in headers {
///             req = req.header(k, v);
///         }
///         let resp = req.body(body.to_vec()).send().await?;
///         Ok(glm_vision_rs::HttpResponse {
///             status: resp.status().as_u16(),
///             body: resp.text().await?,
///         })
///     }
/// }
/// ```
pub trait HttpClient: Send + Sync {
    fn post(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: &[u8],
    ) -> impl Future<Output = Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>>> + Send;
}
