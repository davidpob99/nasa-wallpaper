# nasa-wallpaper

![AUR Version](https://img.shields.io/aur/version/nasa-wallpaper)
![GitHub Release](https://img.shields.io/github/v/release/davidpob99/nasa-wallpaper)
![GitHub Downloads](https://img.shields.io/github/downloads/davidpob99/nasa-wallpaper/total)
![License](https://img.shields.io/github/license/davidpob99/nasa-wallpaper)

A lightweight tool to automatically set your desktop wallpaper with stunning NASA images.
You can choose images from:

* [APOD (Astronomical Picture of the Day)](https://apod.nasa.gov/apod/)
* [NASA Image Library](https://images.nasa.gov/)
* [NASA on Unsplash](https://unsplash.com/@nasa)

![Example](https://images-assets.nasa.gov/image/iss040e008244/iss040e008244~small.jpg)

---

## 🌍 Supported Platforms

* **Windows**
* **macOS**
* **GNOME**, **KDE**, **Cinnamon**, **Unity**, **Budgie**, **XFCE**, **LXDE**, **MATE**, **Deepin**
* **Wayland** (set only, requires `swaybg`)
* **i3** (set only, requires `feh`)

---

## ⚡ Installation

Download the executable that matches your operating system and architecture from the [Releases](https://github.com/davidpob99/nasa-wallpaper/releases) page.
Open a terminal and simply run it.

On Arch Linux, you can install it directly from the [AUR](https://aur.archlinux.org/packages/nasa-wallpaper/).

---

## 🔧 Build from Source

Requirements: [Rust](https://www.rust-lang.org/) **2021 edition or newer**.

Clone the repository and run:

```bash
cargo build --release
```

The compiled binary will be available under `target/release/`.

---

## 🚀 Quick Usage

* Set today’s APOD as wallpaper:

  ```bash
  nasa-wallpaper -a
  ```

* Set the APOD for **March 27, 1999**:

  ```bash
  nasa-wallpaper -a -d 1999-03-27
  ```

* Set a random image from the NASA Image Library:

  ```bash
  nasa-wallpaper -n
  ```

* Set a random image with the keyword *earth*:

  ```bash
  nasa-wallpaper -n -q earth
  ```

* Show help:

  ```bash
  nasa-wallpaper --help
  nasa-wallpaper -h
  ```

📖 **Full documentation:** [Wiki – Command Line Help](https://github.com/davidpob99/nasa-wallpaper/wiki/Command%E2%80%90Line-Help)

---

## 🤝 Contributing

Contributions are welcome! 🎉

If you’d like to contribute:

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Commit your changes (`git commit -m 'Add your feature'`)
4. Push the branch (`git push origin feature/your-feature`)
5. Open a Pull Request

Bug reports, feature requests, and improvements are also welcome via the [Issues](https://github.com/davidpob99/nasa-wallpaper/issues) page.

---

## 📜 License

This project is licensed under the [APACHE 2.0 License](LICENSE).
You are free to use, modify, and distribute it with attribution.
