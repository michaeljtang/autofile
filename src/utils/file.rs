/// A collection of utility functions for file operations.

use std::path::Path;

pub fn is_hidden_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}