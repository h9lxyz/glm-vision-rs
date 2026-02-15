# glm-vision-rs

Rust client for GLM-4.6V (Zhipu AI, Z.AI) vision model analysis. Inspired by the official [Z.AI MCP Server](https://docs.z.ai/devpack/mcp/zread-mcp-server).

> **Note:** This is an unofficial, community-built library. It is not affiliated with, endorsed by, or sponsored by Zhipu AI or Z.AI.

## Background

I wanted to use GLM-4.6V vision capabilities in my [Z.AI coding plan](https://docs.z.ai/devpack/mcp/zread-mcp-server) but didn't want to run the official npm-based MCP server just for that. The system prompts and tool designs in this crate are ported from the [`@z_ai/mcp-server`](https://www.npmjs.com/package/@z_ai/mcp-server?activeTab=code) npm package into a standalone Rust library with no Node.js dependency.

## Tools

Each tool function in `glm_vision::tools` automatically uses the matching system prompt from `glm_vision::prompts`. The prompts are also exposed as public constants if you want to override them via `client.completion()` or `client.completion_raw()` directly.

| Tool | Prompt (auto) | Description |
|------|---------------|-------------|
| `analyze_image` | `GENERAL_IMAGE_ANALYSIS` | General-purpose image description and analysis |
| `extract_text` | `TEXT_EXTRACTION` | Extract text, code, logs from screenshots |
| `diagnose_error` | `ERROR_DIAGNOSIS` | Diagnose errors with root cause and fix suggestions |
| `understand_diagram` | `DIAGRAM_UNDERSTANDING` | Analyze UML, flowcharts, ER, sequence diagrams |
| `analyze_data_viz` | `DATA_VIZ_ANALYSIS` | Analyze charts, graphs, dashboards |
| `ui_diff_check` | `UI_DIFF_CHECK` | Compare two UI screenshots for visual regression |
| `ui_to_artifact` | `UI_TO_ARTIFACT_CODE` | Convert UI screenshot to code (default) |
| | `UI_TO_ARTIFACT_PROMPT` | Convert UI screenshot to an LLM prompt (`output_type: "prompt"`) |
| | `UI_TO_ARTIFACT_SPEC` | Convert UI screenshot to a technical spec (`output_type: "spec"`) |
| | `UI_TO_ARTIFACT_DESCRIPTION` | Convert UI screenshot to a text description (`output_type: "description"`) |
| `analyze_video` | `VIDEO_ANALYSIS` | Analyze video content |

`ui_to_artifact` defaults to generating code. Pass `output_type` to select a different variant: `"prompt"`, `"spec"`, or `"description"`.

## Providers

Three built-in providers are supported. If none is configured, you must set `base_url` manually.

| Provider | Endpoint | Description |
|----------|----------|-------------|
| `Zhipu` | `https://open.bigmodel.cn/api/paas/v4/` | [Zhipu Open Platform](https://open.bigmodel.cn/) |
| `Zai` | `https://api.z.ai/api/paas/v4/` | [Z.AI API](https://api.z.ai/) |
| `ZaiCoding` | `https://api.z.ai/api/coding/paas/v4/` | [Z.AI Coding Plan](https://docs.z.ai/devpack/mcp/zread-mcp-server) |

## Usage

### HTTP Client

This library does not bundle an HTTP client. You provide your own by implementing the `HttpClient` trait, which lets you use whatever HTTP crate (and version) your project already depends on.

```rust
use glm_vision_rs::{HttpClient, HttpResponse};

struct ReqwestClient(reqwest::Client);

impl HttpClient for ReqwestClient {
    async fn post(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: &[u8],
    ) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut req = self.0.post(url);
        for &(k, v) in headers {
            req = req.header(k, v);
        }
        let resp = req.body(body.to_vec()).send().await?;
        Ok(HttpResponse {
            status: resp.status().as_u16(),
            body: resp.text().await?,
        })
    }
}
```

### Setup

```rust
use glm_vision_rs::{Provider, VisionClient, VisionConfig};

let config = VisionConfig::new("your-api-key")
    .with_provider(Provider::ZaiCoding);

let http = ReqwestClient(
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .build()?,
);

let client = VisionClient::new(config, http);
```

### Configuration

```rust
let config = VisionConfig::new("your-api-key")
    .with_provider(Provider::Zhipu)     // or Provider::Zai, Provider::ZaiCoding
    .with_model("glm-4.6v")            // default
    .with_temperature(0.8)              // default
    .with_thinking(true);               // default: enables reasoning mode

// Or use a custom endpoint instead of a provider:
let config = VisionConfig::new("your-api-key")
    .with_base_url("https://custom.example.com/v1/");
```

### Analyze an image

Works with URLs or local file paths:

```rust
let result = glm_vision::tools::analyze_image(
    &client,
    "https://example.com/photo.jpg",  // or "/path/to/local.png"
    "Describe what you see in this image.",
)
.await?;
```

### Extract text from a screenshot

```rust
let result = glm_vision::tools::extract_text(
    &client,
    "/path/to/screenshot.png",
    "Extract all visible code from this screenshot.",
    Some("rust"),  // optional: programming language hint
)
.await?;
```

### Diagnose an error

```rust
let result = glm_vision::tools::diagnose_error(
    &client,
    "/path/to/error.png",
    "What is this error and how do I fix it?",
    Some("Running cargo build"),  // optional: context
)
.await?;
```

### Analyze a diagram

```rust
let result = glm_vision::tools::understand_diagram(
    &client,
    "/path/to/diagram.png",
    "Explain this diagram in detail.",
    Some("sequence"),  // optional: diagram type hint
)
.await?;
```

### Analyze a data visualization

```rust
let result = glm_vision::tools::analyze_data_viz(
    &client,
    "/path/to/dashboard.png",
    "What trends do you see in this data?",
    Some("trends"),  // optional: analysis focus
)
.await?;
```

### Compare two UI screenshots

```rust
let result = glm_vision::tools::ui_diff_check(
    &client,
    "/path/to/expected.png",
    "/path/to/actual.png",
    "List all visual differences between these two screenshots.",
)
.await?;
```

### Convert UI to artifact

Defaults to code generation. Pass `output_type` to select a different variant:

```rust
// Generate HTML/CSS code (default)
let code = glm_vision::tools::ui_to_artifact(
    &client,
    "/path/to/ui.png",
    None,  // defaults to "code"
    "Generate responsive HTML/CSS for this design.",
)
.await?;

// Generate a technical spec
let spec = glm_vision::tools::ui_to_artifact(
    &client,
    "/path/to/ui.png",
    Some("spec"),  // or "prompt", "description"
    "Write a technical specification for this UI.",
)
.await?;
```

### Analyze a video

```rust
let result = glm_vision::tools::analyze_video(
    &client,
    "/path/to/video.mp4",
    "Describe what happens in this video.",
)
.await?;
```

### Advanced: raw JSON response

Use `client.completion_raw()` with any prompt for direct access to the API response:

```rust
let image = client.process_image("/path/to/image.png")?;
let raw_json = client
    .completion_raw(
        glm_vision::prompts::GENERAL_IMAGE_ANALYSIS,
        vec![image],
        "Describe this image.",
    )
    .await?;
```

## Example Results

See [examples/EXAMPLES.md](examples/EXAMPLES.md) for full output from all 11 tools with token counts, timings, and input images.

## License

MIT
