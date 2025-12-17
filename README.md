# AutoFile - Smart File Organizer

A Rust-based daemon that automatically organizes files in your Downloads folder (or any specified directory) by detecting file types and moving them to appropriate locations.

## Features

- **Intelligent File Detection**: Uses magic bytes (file signatures) for accurate file type detection, with extension fallback
- **Real-time Monitoring**: Watches directories for new files using inotify (Linux) / FSEvents (macOS)
- **Smart Categorization**: Automatically categorizes files into:
  - Documents (PDFs, Word docs, etc.) → `~/Documents`
  - Images (PNG, JPG, etc.) → `~/Pictures`
  - Videos (MP4, AVI, etc.) → `~/Videos`
  - Audio (MP3, FLAC, etc.) → `~/Music`
  - Archives (ZIP, TAR, etc.) → `~/Documents/Archives`
  - Code files (.rs, .py, etc.) → `~/Projects`
- **Conflict Resolution**: Automatically handles duplicate file names by appending numbers
- **Safe Operations**: Uses atomic operations and handles cross-filesystem moves
- **Comprehensive Logging**: Track all file operations with detailed logs

## Installation

### Prerequisites

- Rust 1.70 or newer
- Cargo

### Build from Source

```bash
# Clone or navigate to the project directory
cd autofile

# Build the release version
cargo build --release

# The binary will be available at target/release/autofile
```

## Usage

### Basic Usage

Monitor your Downloads folder (default):

```bash
./target/release/autofile
```

### Monitor a Custom Directory

```bash
./target/release/autofile /path/to/directory
```

### Run with Verbose Logging

```bash
RUST_LOG=debug ./target/release/autofile
```

### Run as a Background Daemon

#### On macOS/Linux (using nohup):

```bash
nohup ./target/release/autofile > autofile.log 2>&1 &
```

#### Using systemd (Linux):

Create a systemd service file at `~/.config/systemd/user/autofile.service`:

```ini
[Unit]
Description=AutoFile - Smart File Organizer
After=network.target

[Service]
Type=simple
ExecStart=/path/to/autofile/target/release/autofile
Restart=on-failure
Environment="RUST_LOG=info"

[Install]
WantedBy=default.target
```

Then enable and start the service:

```bash
systemctl --user enable autofile
systemctl --user start autofile
systemctl --user status autofile
```

#### Using launchd (macOS):

Create a plist file at `~/Library/LaunchAgents/com.autofile.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.autofile</string>
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/autofile/target/release/autofile</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/autofile.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/autofile.error.log</string>
</dict>
</plist>
```

Then load the service:

```bash
launchctl load ~/Library/LaunchAgents/com.autofile.plist
launchctl start com.autofile
```

## File Categories

| Category | Extensions | Destination |
|----------|-----------|-------------|
| Documents | pdf, doc, docx, txt, rtf, odt, xls, xlsx, ppt, pptx | ~/Documents |
| Images | jpg, jpeg, png, gif, bmp, svg, webp, ico, tiff | ~/Pictures |
| Videos | mp4, avi, mkv, mov, wmv, flv, webm, m4v | ~/Videos |
| Audio | mp3, wav, flac, aac, ogg, m4a, wma | ~/Music |
| Archives | zip, rar, 7z, tar, gz, bz2, xz | ~/Documents/Archives |
| Code | rs, py, js, ts, go, java, c, cpp, html, css, json, yaml, md, etc. | ~/Projects |

## Project Structure

```
autofile/
├── src/
│   ├── main.rs              # Application entry point
│   ├── organizer.rs         # Main file organization logic
│   └── modules/
│       ├── mod.rs
│       ├── watcher.rs       # File system watching
│       ├── detector.rs      # File type detection
│       ├── categorizer.rs   # Categorization rules
│       └── mover.rs         # Safe file moving
├── Cargo.toml
└── README.md
```

## How It Works

1. **Watching**: The application monitors the specified directory for new files using the `notify` crate
2. **Detection**: When a file appears, it reads the file's magic bytes to determine its type
3. **Categorization**: Based on the detected type, it looks up the appropriate destination folder
4. **Moving**: The file is safely moved to the destination, with automatic conflict resolution if needed
5. **Logging**: All operations are logged for transparency and debugging

## Logging Levels

Control logging verbosity with the `RUST_LOG` environment variable:

- `error`: Only errors
- `warn`: Warnings and errors
- `info`: General information (default)
- `debug`: Detailed debugging information
- `trace`: Very verbose debugging

Example:
```bash
RUST_LOG=debug ./target/release/autofile
```

## Safety Features

- **Magic Byte Detection**: Doesn't rely solely on file extensions, which can be misleading
- **Atomic Operations**: File moves are atomic when possible
- **Conflict Resolution**: Automatically renames files to prevent overwrites
- **Cross-filesystem Support**: Falls back to copy+delete for moves across filesystems
- **Error Recovery**: Continues operation even if individual file operations fail

## Troubleshooting

### Permission Denied

Make sure the application has read/write permissions for both the source and destination directories.

### Files Not Being Detected

- Check that the directory exists and is readable
- Verify file permissions
- Check logs with `RUST_LOG=debug`

### Daemon Not Starting

- Verify the binary path in the service configuration
- Check system logs: `journalctl --user -u autofile` (systemd) or `log show --predicate 'process == "autofile"'` (macOS)

## Contributing

Feel free to submit issues or pull requests to improve AutoFile!

## License

This project is open source and available under the MIT License.