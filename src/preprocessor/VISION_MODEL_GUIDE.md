# Vision Model Implementation Guide

This guide explains how to implement the AI-powered image renaming using candle-rs.

## Current Status

The `ImageRenamer` preprocessor is **scaffolded but not yet functional**. The vision model integration is stubbed out and needs to be implemented.

## Recommended Models

For image captioning with candle-rs, consider these models:

### 1. Moondream (Recommended - Lightweight)
- **Size**: ~1.6GB
- **Speed**: Fast inference on CPU
- **Quality**: Good for general scenes
- **Model**: `vikhyatk/moondream2`

### 2. BLIP-2
- **Size**: ~2-4GB depending on variant
- **Speed**: Medium
- **Quality**: Excellent captions
- **Model**: `Salesforce/blip2-opt-2.7b`

### 3. LLaVA
- **Size**: 4-13GB depending on variant
- **Speed**: Slower but more detailed
- **Quality**: Very detailed descriptions
- **Model**: `liuhaotian/llava-v1.5-7b`

## Implementation Steps

### Step 1: Model Loading

Add model initialization to `ImageRenamer::new()`:

```rust
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::blip; // or moondream
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

pub struct ImageRenamer {
    enabled: bool,
    model: Option<VisionModel>, // Add this
    device: Device,
}

struct VisionModel {
    model: blip::BlipForConditionalGeneration, // or appropriate model type
    tokenizer: Tokenizer,
}

impl ImageRenamer {
    pub fn new() -> Self {
        // Try to load model, fall back to disabled if it fails
        let (enabled, model, device) = match Self::load_model() {
            Ok((m, d)) => (true, Some(m), d),
            Err(e) => {
                log::warn!("Failed to load vision model: {}. Image renaming disabled.", e);
                (false, None, Device::Cpu)
            }
        };

        Self { enabled, model, device }
    }

    fn load_model() -> Result<(VisionModel, Device)> {
        let device = Device::Cpu; // or Device::cuda_if_available(0)?

        // Download model from Hugging Face
        let api = Api::new()?;
        let repo = api.repo(Repo::new(
            "vikhyatk/moondream2".to_string(),
            RepoType::Model,
        ));

        let model_file = repo.get("model.safetensors")?;
        let tokenizer_file = repo.get("tokenizer.json")?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_file)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        // Load model weights
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[model_file], DType::F32, &device)? };
        let model = blip::BlipForConditionalGeneration::load(vb, &config)?;

        Ok((VisionModel { model, tokenizer }, device))
    }
}
```

### Step 2: Image Processing

Implement `generate_descriptive_name()`:

```rust
use image::io::Reader as ImageReader;
use candle_core::DType;

fn generate_descriptive_name(&self, image_path: &Path) -> Result<String> {
    let model = self.model.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Model not loaded"))?;

    // Load and preprocess image
    let img = ImageReader::open(image_path)?
        .decode()?
        .resize_exact(384, 384, image::imageops::FilterType::Triangle)
        .to_rgb8();

    // Convert to tensor
    let img_tensor = self.preprocess_image(&img)?;

    // Generate caption
    let caption = self.generate_caption(&model, img_tensor)?;

    Ok(caption)
}

fn preprocess_image(&self, img: &image::RgbImage) -> Result<Tensor> {
    let (width, height) = img.dimensions();
    let img_data: Vec<f32> = img.pixels()
        .flat_map(|pixel| {
            // Normalize to [0, 1] and apply ImageNet normalization
            let r = (pixel[0] as f32 / 255.0 - 0.485) / 0.229;
            let g = (pixel[1] as f32 / 255.0 - 0.456) / 0.224;
            let b = (pixel[2] as f32 / 255.0 - 0.406) / 0.225;
            vec![r, g, b]
        })
        .collect();

    let tensor = Tensor::from_vec(
        img_data,
        (1, 3, height as usize, width as usize),
        &self.device,
    )?;

    Ok(tensor)
}

fn generate_caption(&self, model: &VisionModel, img_tensor: Tensor) -> Result<String> {
    // Run model inference
    let output = model.model.generate(&img_tensor, /* max_length */ 50)?;

    // Decode tokens to text
    let caption = model.tokenizer
        .decode(&output.to_vec1::<u32>()?, true)
        .map_err(|e| anyhow::anyhow!("Failed to decode: {}", e))?;

    Ok(caption)
}
```

### Step 3: Configuration

Add configuration options in `config.toml`:

```toml
[preprocessor.image_renamer]
enabled = false  # Set to true to enable
model = "vikhyatk/moondream2"  # Model to use
device = "cpu"  # or "cuda"
max_filename_length = 50
```

Update `Config` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessorConfig {
    #[serde(default)]
    pub image_renamer: ImageRenamerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRenamerConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_device")]
    pub device: String,
    #[serde(default = "default_max_length")]
    pub max_filename_length: usize,
}

fn default_model() -> String { "vikhyatk/moondream2".to_string() }
fn default_device() -> String { "cpu".to_string() }
fn default_max_length() -> usize { 50 }
```

## Testing

```rust
#[test]
fn test_image_renaming() {
    let renamer = ImageRenamer::new();
    if !renamer.enabled {
        println!("Skipping test - model not available");
        return;
    }

    let test_image = Path::new("test_data/IMG_1234.jpg");
    let result = renamer.process(test_image).unwrap();

    assert_ne!(result, test_image);
    assert!(!result.file_name().unwrap().to_str().unwrap().starts_with("IMG_"));
}
```

## Performance Considerations

1. **Lazy Loading**: Only load model when first image is encountered
2. **Batching**: Process multiple images at once if many arrive together
3. **Caching**: Cache model in memory between file operations
4. **GPU Acceleration**: Use CUDA if available for faster processing

## Error Handling

- If model fails to load, disable the preprocessor gracefully
- If inference fails on a specific image, log warning and keep original name
- Always return a valid path even if renaming fails

## Example Output

```
Before: IMG_1234.jpg
After:  sunset_over_ocean_1734567890.jpg

Before: DSC_5678.png
After:  cat_sitting_on_windowsill_1734567891.png
```
