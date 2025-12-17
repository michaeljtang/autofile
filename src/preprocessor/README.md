# Preprocessor Module

The preprocessor module provides a plugin-like architecture for transforming files before they are organized.

## Architecture

- **`Preprocessor` trait**: Defines the interface all preprocessors must implement
- **`PreprocessorPipeline`**: Manages and applies multiple preprocessors in sequence
- Each preprocessor is a separate file in this module

## Built-in Preprocessors

### HEIC Converter (`heic_converter.rs`)
Converts HEIC/HEIF images to PNG format before organization.

- **Trigger**: Files with `.heic` or `.heif` extensions
- **Action**: Converts to PNG, deletes original HEIC file
- **Requirements**:
  - macOS: Uses built-in `sips` command
  - Other platforms: Requires ImageMagick (`convert` command)

## Adding a New Preprocessor

### Step 1: Create the preprocessor file

Create a new file in `src/preprocessor/` (e.g., `webp_converter.rs`):

```rust
use super::Preprocessor;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct WebpConverter;

impl WebpConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Preprocessor for WebpConverter {
    fn name(&self) -> &str {
        "WebP to PNG Converter"
    }

    fn should_process(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            ext.to_str().unwrap_or("").to_lowercase() == "webp"
        } else {
            false
        }
    }

    fn process(&self, path: &Path) -> Result<PathBuf> {
        // Your conversion logic here
        // Return the new path (or original if unchanged)
        Ok(path.to_path_buf())
    }
}
```

### Step 2: Add the module to `mod.rs`

At the top of `mod.rs`, add:
```rust
mod webp_converter;
pub use webp_converter::WebpConverter;
```

### Step 3: Add to the pipeline

In `PreprocessorPipeline::new()`, add:
```rust
preprocessors.push(Box::new(WebpConverter::new()));
```

## Processing Order

Preprocessors are applied in the order they are added to the pipeline in `PreprocessorPipeline::new()`. Each preprocessor receives the output path from the previous one.

## Example Use Cases

- **Format Conversion**: HEIC → PNG, WebP → PNG, RAW → JPG
- **Compression**: Reduce file size while maintaining quality
- **Metadata Removal**: Strip EXIF data for privacy
- **Renaming**: Normalize file names based on patterns
- **Watermarking**: Add watermarks to images
- **Deduplication**: Hash and skip duplicate files
