use anyhow::{bail, Result};

use crate::client::VisionClient;
use crate::prompts;

const DEFAULT_RETRIES: u32 = 2;

/// Convert a UI screenshot into code, prompt, spec, or description.
///
/// `output_type` selects the system prompt variant:
/// - `None` or `Some("code")` — generate code (default)
/// - `Some("prompt")` — generate an LLM prompt
/// - `Some("spec")` — generate a technical spec
/// - `Some("description")` — generate a text description
pub async fn ui_to_artifact(
    client: &VisionClient,
    image_source: &str,
    output_type: Option<&str>,
    prompt: &str,
) -> Result<String> {
    let ot = output_type.unwrap_or("code");
    let system_prompt = prompts::ui_to_artifact_prompt(ot).ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid output_type '{}'. Must be one of: code, prompt, spec, description",
            ot
        )
    })?;

    let image = client.process_image(image_source)?;
    client
        .completion_with_retry(system_prompt, vec![image], prompt, DEFAULT_RETRIES)
        .await
}

/// Extract text from a screenshot (code, logs, configuration, documentation).
///
/// Optionally specify `programming_language` to enhance extraction accuracy.
pub async fn extract_text(
    client: &VisionClient,
    image_source: &str,
    prompt: &str,
    programming_language: Option<&str>,
) -> Result<String> {
    let image = client.process_image(image_source)?;
    let enhanced_prompt = match programming_language {
        Some(lang) => format!(
            "Programming language context: {}\n\n{}",
            lang, prompt
        ),
        None => prompt.to_string(),
    };
    client
        .completion_with_retry(prompts::TEXT_EXTRACTION, vec![image], &enhanced_prompt, DEFAULT_RETRIES)
        .await
}

/// Diagnose an error from a screenshot.
///
/// Optionally provide `context` (e.g., what the user was doing when the error occurred).
pub async fn diagnose_error(
    client: &VisionClient,
    image_source: &str,
    prompt: &str,
    context: Option<&str>,
) -> Result<String> {
    let image = client.process_image(image_source)?;
    let enhanced_prompt = match context {
        Some(ctx) => format!(
            "Context: {}\n\n{}",
            ctx, prompt
        ),
        None => prompt.to_string(),
    };
    client
        .completion_with_retry(prompts::ERROR_DIAGNOSIS, vec![image], &enhanced_prompt, DEFAULT_RETRIES)
        .await
}

/// Analyze and explain a technical diagram.
///
/// Optionally specify `diagram_type` (e.g., "UML class", "sequence", "ER", "flowchart").
pub async fn understand_diagram(
    client: &VisionClient,
    image_source: &str,
    prompt: &str,
    diagram_type: Option<&str>,
) -> Result<String> {
    let image = client.process_image(image_source)?;
    let enhanced_prompt = match diagram_type {
        Some(dt) => format!(
            "Diagram type: {}\n\n{}",
            dt, prompt
        ),
        None => prompt.to_string(),
    };
    client
        .completion_with_retry(prompts::DIAGRAM_UNDERSTANDING, vec![image], &enhanced_prompt, DEFAULT_RETRIES)
        .await
}

/// Analyze a data visualization (chart, graph, dashboard).
///
/// Optionally specify `analysis_focus` (e.g., "trends", "anomalies", "comparison").
pub async fn analyze_data_viz(
    client: &VisionClient,
    image_source: &str,
    prompt: &str,
    analysis_focus: Option<&str>,
) -> Result<String> {
    let image = client.process_image(image_source)?;
    let enhanced_prompt = match analysis_focus {
        Some(focus) => format!(
            "Analysis focus: {}\n\n{}",
            focus, prompt
        ),
        None => prompt.to_string(),
    };
    client
        .completion_with_retry(prompts::DATA_VIZ_ANALYSIS, vec![image], &enhanced_prompt, DEFAULT_RETRIES)
        .await
}

/// Compare two UI screenshots (expected vs actual) for visual regression.
pub async fn ui_diff_check(
    client: &VisionClient,
    expected: &str,
    actual: &str,
    prompt: &str,
) -> Result<String> {
    let expected_img = client.process_image(expected)?;
    let actual_img = client.process_image(actual)?;

    let enhanced_prompt = format!(
        "<images>\nThe first image is the EXPECTED/REFERENCE design (the target).\n\
         The second image is the ACTUAL/CURRENT implementation (what needs to be checked).\n\
         </images>\n\n{}",
        prompt
    );

    client
        .completion_with_retry(
            prompts::UI_DIFF_CHECK,
            vec![expected_img, actual_img],
            &enhanced_prompt,
            DEFAULT_RETRIES,
        )
        .await
}

/// General-purpose image analysis.
pub async fn analyze_image(
    client: &VisionClient,
    image_source: &str,
    prompt: &str,
) -> Result<String> {
    if prompt.trim().is_empty() {
        bail!("Prompt is required for image analysis");
    }
    let image = client.process_image(image_source)?;
    client
        .completion_with_retry(prompts::GENERAL_IMAGE_ANALYSIS, vec![image], prompt, DEFAULT_RETRIES)
        .await
}

/// Analyze video content.
pub async fn analyze_video(
    client: &VisionClient,
    video_source: &str,
    prompt: &str,
) -> Result<String> {
    if prompt.trim().is_empty() {
        bail!("Prompt is required for video analysis");
    }
    let video = client.process_video(video_source)?;
    client
        .completion_with_retry(prompts::VIDEO_ANALYSIS, vec![video], prompt, DEFAULT_RETRIES)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts;

    #[test]
    fn test_ui_to_artifact_prompt_selection() {
        // Verify all output types resolve to non-empty prompts
        for t in &["code", "prompt", "spec", "description"] {
            let p = prompts::ui_to_artifact_prompt(t);
            assert!(p.is_some(), "missing prompt for output_type={}", t);
            assert!(!p.unwrap().is_empty());
        }
    }

    #[test]
    fn test_prompt_enhancement_with_language() {
        let base = "Extract the code from this screenshot";
        let lang = "python";
        let enhanced = format!("Programming language context: {}\n\n{}", lang, base);
        assert!(enhanced.contains("python"));
        assert!(enhanced.contains(base));
    }

    #[test]
    fn test_prompt_enhancement_with_context() {
        let base = "What is this error?";
        let ctx = "Running cargo build on a fresh clone";
        let enhanced = format!("Context: {}\n\n{}", ctx, base);
        assert!(enhanced.contains(ctx));
        assert!(enhanced.contains(base));
    }

    #[test]
    fn test_prompt_enhancement_with_diagram_type() {
        let base = "Explain this diagram";
        let dt = "UML class";
        let enhanced = format!("Diagram type: {}\n\n{}", dt, base);
        assert!(enhanced.contains("UML class"));
    }

    #[test]
    fn test_ui_diff_prompt_enhancement() {
        let prompt = "Compare these two designs";
        let enhanced = format!(
            "<images>\nThe first image is the EXPECTED/REFERENCE design (the target).\n\
             The second image is the ACTUAL/CURRENT implementation (what needs to be checked).\n\
             </images>\n\n{}",
            prompt
        );
        assert!(enhanced.contains("EXPECTED/REFERENCE"));
        assert!(enhanced.contains("ACTUAL/CURRENT"));
        assert!(enhanced.contains(prompt));
    }
}
