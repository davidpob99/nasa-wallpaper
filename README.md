# 🚀 nasa-wallpaper
![GitHub Release](https://img.shields.io/github/v/release/davidpob99/nasa-wallpaper)
![Crates.io Version](https://img.shields.io/crates/v/nasa-wallpaper)
![License](https://img.shields.io/github/license/davidpob99/nasa-wallpaper)

A professional, lightweight tool to automatically set your desktop wallpaper with stunning NASA images. Refactored for extreme robustness and modern Rust standards.

## ✨ Sources
- 🌌 **APOD**: [Astronomy Picture of the Day](https://apod.nasa.gov/apod/)
- 🔍 **NASA Image Library**: Massive searchable archive at [images.nasa.gov](https://images.nasa.gov/)
- 📸 **NASA Unsplash**: Curated high-res images from [Unsplash/@nasa](https://unsplash.com/@nasa)

---

## 🛠️ Key Improvements in v2.2.0
- **Modular Architecture**: Clean, maintainable codebase split into specialized modules.
- **Robust Error Handling**: Powered by `anyhow`, ensuring the app never crashes silently.
- **Improved Search**: Enhanced NASA Image Library search with proper URL encoding and randomized results.
- **Modern Stack**: Updated to latest dependencies (`clap` v4, `reqwest` v0.12, `serde`).
- **Safe Downloads**: Implemented secure temporary file handling for cross-platform reliability.

---

## 🔑 NASA API Key (Recommended)

While the app works with the default `DEMO_KEY`, it has strict rate limits and may timeout. **Get your free personal API key** for better reliability:

1. Visit [https://api.nasa.gov/](https://api.nasa.gov/)
2. Fill out the simple registration form (name + email)
3. Receive your API key instantly via email

### Using Your API Key

**Option 1: Environment Variable (Recommended)**
```bash
# Linux/macOS
export NASA_API_KEY="your_api_key_here"

# Windows (PowerShell)
$env:NASA_API_KEY="your_api_key_here"

# Windows (CMD)
set NASA_API_KEY=your_api_key_here
```

**Option 2: Command Line Argument**
```bash
nasa-wallpaper apod --key your_api_key_here
```

---

## ⚡ Quick Usage

Set today’s **APOD** as wallpaper:
```bash
nasa-wallpaper apod
```

Set the **APOD** for a specific date:
```bash
nasa-wallpaper apod --date 2023-12-25
```

Set a random image from the **NASA Image Library**:
```bash
nasa-wallpaper nasa_image
```

Search for a specific topic (e.g., *Mars*):
```bash
nasa-wallpaper nasa_image --query "Mars"
```

Get a random image from **Unsplash**:
```bash
nasa-wallpaper unsplash
```

---

## 🔧 Build from Source
Requirements: [Rust](https://www.rust-lang.org/) **2021 edition or newer**.

```bash
git clone https://github.com/davidpob99/nasa-wallpaper
cd nasa-wallpaper
cargo build --release
```

---

## 🤝 Contributing
Contributions are what make the open source community such an amazing place to learn, inspire, and create.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

---

## 📜 License
Distributed under the **Apache 2.0 License**. See `LICENSE` for more information.
