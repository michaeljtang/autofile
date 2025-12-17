# Implementing the Vision Model for Image Renaming

The image renaming preprocessor is scaffolded but needs a vision model implementation. Here's how to do it.

## Quick Start

The candle-rs ecosystem doesn't have ready-to-use vision-language models like BLIP or Moondream with simple APIs yet. You have a few options:

### Option 1: Use an External API (Easiest)

Instead of running models locally, use a vision API:

**OpenAI Vision API:**
```rust
// Add dependencies
// openai-api-rs = "5.0"

use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{ChatCompletionRequest, MessageRole};

async fn generate_caption_openai(image_path: &Path, api_key: &str) -> Result<String> {
    let client = Client::new(api_key.to_string());

    // Encode image to base64
    let image_data = std::fs::read(image_path)?;
    let base64_image = base64::encode(&image_data);

    let request = ChatCompletionRequest::new(
        "gpt-4-vision-preview".to_string(),
        vec![Message {
            role: MessageRole::User,
            content: format!("Describe this image in 5 words or less. Image: data:image/jpeg;base64,{}", base64_image),
        }],
    );

    let response = client.chat_completion(request).await?;
    Ok(response.choices[0].message.content.clone())
}
```

**Pros:** Simple, accurate, no local compute
**Cons:** Costs money, requires internet, slower

### Option 2: Use ONNX Runtime (Recommended)

Use pre-exported ONNX models with `ort`:

```toml
# Add to Cargo.toml
ort = "2.0"
image = "0.25"
ndarray = "0.16"
```

```rust
use ort::{GraphOptimizationLevel, Session};
use image::GenericImageView;
use ndarray::{Array, IxDyn};

struct VisionModel {
    session: Session,
}

impl VisionModel {
    fn new() -> Result<Self> {
        // Download BLIP model from Hugging Face (ONNX format)
        let model_path = download_onnx_model()?;

        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)?;

        Ok(Self { session })
    }

    fn caption(&self, image_path: &Path) -> Result<String> {
        // Load and preprocess image
        let img = image::open(image_path)?;
        let img_array = preprocess_image(&img);

        // Run inference
        let outputs = self.session.run(ort::inputs!["pixel_values" => img_array]?)?;
        let caption = decode_output(&outputs)?;

        Ok(caption)
    }
}
```

**Pros:** Fast, offline, free
**Cons:** Need to find/convert ONNX models

### Option 3: Use a Simpler Rust ML Library

Use `tract` with ONNX models:

```toml
tract-onnx = "0.21"
```

Similar approach to Option 2 but with tract instead of ort.

### Option 4: Just Use CLI Tools (Simplest)

Call external vision tools:

```rust
use std::process::Command;

fn generate_caption_cli(image_path: &Path) -> Result<String> {
    // Use llava.cpp, ollama, or other local tools
    let output = Command::new("ollama")
        .arg("run")
        .arg("llava")
        .arg(format!("describe this image briefly: {}", image_path.display()))
        .output()?;

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
```

**Requirements:** Install ollama or llava.cpp locally
**Pros:** Very simple, leverages existing tools
**Cons:** External dependency, slower

## Recommended Approach

For a quick win, I recommend **Option 4** with ollama:

1. Install ollama: `brew install ollama` (macOS) or from ollama.ai
2. Pull a vision model: `ollama pull llava`
3. Implement the CLI integration shown above

## Full Implementation Example (Option 4)

```rust
// In image_renamer.rs

impl ImageRenamer {
    pub fn new() -> Self {
        // Check if ollama is available
        let enabled = Command::new("ollama")
            .arg("list")
            .output()
            .is_ok();

        if enabled {
            log::info!("Image renamer enabled (using ollama)");
        } else {
            log::info!("Image renamer disabled (ollama not found)");
        }

        Self { enabled }
    }

    fn generate_descriptive_name(&self, image_path: &Path) -> Result<String> {
        let output = Command::new("ollama")
            .arg("run")
            .arg("llava")
            .arg(format!("describe this image in 5 words or less: {}", image_path.display()))
            .output()
            .context("Failed to run ollama")?;

        if !output.status.success() {
            anyhow::bail!("ollama command failed");
        }

        let caption = String::from_utf8(output.stdout)?
            .trim()
            .to_string();

        Ok(caption)
    }
}
```

## Testing

```bash
# Install ollama
brew install ollama

# Start ollama service
ollama serve

# Pull vision model
ollama pull llava

# Test manually
ollama run llava "describe this image briefly: /path/to/IMG_1234.jpg"

# Run autofile
cargo run ~/Downloads
```

## Performance Notes

- **ollama/llava:** ~2-5 seconds per image on M1 Mac
- **OpenAI API:** ~1-2 seconds per image (network dependent)
- **ONNX with ort:** ~0.5-1 second per image (optimized)

Choose based on your needs:
- **Speed:** ONNX
- **Accuracy:** OpenAI API
- **Simplicity:** ollama
