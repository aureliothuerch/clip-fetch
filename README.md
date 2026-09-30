# clipfetch

Download TikTok, Instagram and YouTube videos from the terminal. Pick the quality from a menu, up to 4K and 60fps where the site offers it. Nothing else needs to be installed.

## Install

macOS / Linux:

```
curl -LsSf https://github.com/aureliothuerch/clip-fetch/releases/latest/download/clipfetch-installer.sh | sh
```

Windows (PowerShell):

```
irm https://github.com/aureliothuerch/clip-fetch/releases/latest/download/clipfetch-installer.ps1 | iex
```

With Rust installed:

```
cargo install clipfetch
```

## Usage

```
cf "https://www.youtube.com/watch?v=..."
cf "https://vm.tiktok.com/..." -q 720
cf "https://www.youtube.com/watch?v=..." -o ~/Videos
```

- `-q, --quality`: maximum resolution, skips the menu
- `-o, --output`: output folder, default is your Downloads folder

The first run downloads yt-dlp and ffmpeg into your cache folder.