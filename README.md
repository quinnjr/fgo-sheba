# 🔮 FGO Sheba

<div align="center">

![Sheba Logo](android/app/src/main/res/drawable/ic_sheba_logo.xml)

**AI-Powered Fate/Grand Order Automation for Android**

*The Lens of the Future - シェバ*

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Kotlin](https://img.shields.io/badge/Kotlin-7F52FF?style=for-the-badge&logo=kotlin&logoColor=white)](https://kotlinlang.org/)
[![Android](https://img.shields.io/badge/Android-3DDC84?style=for-the-badge&logo=android&logoColor=white)](https://developer.android.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-gold.svg?style=for-the-badge)](LICENSE)

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Architecture](#-architecture) • [Contributing](#-contributing)

</div>

---

## 🌟 Features

### 🤖 AI Battle Engine
- **Intelligent Card Selection** - ML-powered command card chain optimization
- **NP Timing** - Smart Noble Phantasm usage based on enemy HP and wave analysis
- **Skill Management** - Automatic skill usage with proper timing and targeting
- **Enemy Prioritization** - Strategic targeting based on class advantage and threat level

### 👁️ Vision System
- **Screen Recognition** - ONNX-based ML models for real-time game state detection
- **Card Classification** - Automatic Buster/Arts/Quick card type recognition
- **Servant Detection** - Identify active servants and their positions
- **HP/NP Bar Reading** - OCR-based gauge monitoring

### 🛡️ Stealth & Anti-Detection
- **Human Behavior Simulation** - Randomized timing and tap positions
- **Detection Avoidance** - Hide from recent apps when FGO is active
- **Generic Service Names** - Innocuous accessibility service labeling
- **Security Warnings** - Alerts for USB debugging, developer options, root detection

### 🎨 Beautiful UI
- **FGO-Inspired Design** - Dark cosmic theme with gold accents
- **Smooth Animations** - Pulse effects, fade-ins, and slide animations
- **Session Statistics** - Real-time battle count, runtime, and NP usage tracking
- **Floating Overlay** - Draggable control panel during automation

### 🌍 Multi-Language Support
17 languages supported:

| Language | Code | | Language | Code |
|----------|------|-|----------|------|
| 🇬🇧 English | `en` | | 🇯🇵 日本語 | `ja` |
| 🇨🇳 简体中文 | `zh-CN` | | 🇹🇼 繁體中文 | `zh-TW` |
| 🇰🇷 한국어 | `ko` | | 🇪🇸 Español | `es` |
| 🇫🇷 Français | `fr` | | 🇩🇪 Deutsch | `de` |
| 🇧🇷 Português | `pt-BR` | | 🇷🇺 Русский | `ru` |
| 🇮🇹 Italiano | `it` | | 🇮🇩 Bahasa Indonesia | `id` |
| 🇹🇭 ภาษาไทย | `th` | | 🇻🇳 Tiếng Việt | `vi` |
| 🇸🇦 العربية | `ar` | | 🇵🇱 Polski | `pl` |
| 🇹🇷 Türkçe | `tr` | | 🇳🇱 Nederlands | `nl` |

---

## 📋 Requirements

- Android 8.0+ (API 26+)
- Fate/Grand Order (JP, NA, KR, or TW version)
- ~100MB storage for ML models

---

## 🚀 Installation

### Pre-built APK
Download the latest release from the [Releases](https://github.com/quinnjr/fgo-sheba/releases) page.

### Build from Source

#### Prerequisites
- Rust 1.70+ with Android NDK targets
- Android Studio / Gradle 8.5+
- JDK 17+

```bash
# Clone the repository
git clone https://github.com/quinnjr/fgo-sheba.git
cd fgo-sheba

# Build Rust library for Android
cargo build --release --target aarch64-linux-android
cargo build --release --target armv7-linux-androideabi

# Build Android APK
cd android
./gradlew assembleRelease
```

---

## 📖 Usage

1. **Install the APK** on your Android device
2. **Grant Permissions**:
   - Enable "Display Helper" in Accessibility Settings
   - Grant overlay (draw over apps) permission
3. **Launch FGO** and navigate to a battle quest
4. **Start Automation** from the Sheba app
5. **Monitor** via the floating overlay control

### Tips for Best Results
- Disable USB debugging while using the app
- Use on a stable network connection
- Ensure FGO is updated to the latest version

---

## 🏗️ Architecture

```
fgo-sheba/
├── src/                    # Rust core library
│   ├── ai/                 # Battle AI engine
│   │   ├── card_selector   # Card chain optimization
│   │   ├── np_timing       # Noble Phantasm decisions
│   │   ├── skill_usage     # Skill timing logic
│   │   └── enemy_priority  # Target selection
│   ├── vision/             # Screen recognition
│   │   ├── models          # ONNX model inference
│   │   ├── recognition     # Card/servant detection
│   │   └── ocr             # HP/NP bar reading
│   ├── game/               # Game state management
│   ├── android/            # JNI bridge
│   └── stealth/            # Anti-detection
├── android/                # Android application
│   └── app/src/main/
│       ├── kotlin/io/sheba/
│       │   ├── MainActivity
│       │   ├── ShebaAccessibilityService
│       │   ├── OverlayService
│       │   └── stealth/    # Human simulation
│       └── res/            # UI resources
├── training/               # ML model training
│   ├── download_assets.py  # Atlas Academy downloader
│   └── train_*.py          # Training scripts
└── models/                 # Pre-trained ONNX models
```

---

## 🔧 Configuration

Settings can be adjusted in-app:

| Setting | Description | Default |
|---------|-------------|---------|
| Card Priority | Preferred card type order | Buster > Arts > Quick |
| NP Threshold | Enemy HP % to trigger NP | 50% |
| Skill Timing | When to use servant skills | Start of wave |
| Stealth Level | Human simulation intensity | Normal |

---

## 🛠️ Development

### Running Tests

```bash
# Rust tests
cargo test

# Android tests
cd android && ./gradlew test
```

### Training Custom Models

```bash
cd training
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Download assets from Atlas Academy
python download_assets.py

# Train card classifier
python train_card_classifier.py
```

---

## ⚠️ Disclaimer

This tool is provided for **educational and research purposes only**.

- Use at your own risk
- The developers are not responsible for any account actions taken by game publishers
- This project is not affiliated with or endorsed by Aniplex, TYPE-MOON, or Lasengle

---

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) first.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 💖 Support

If you find this project useful, consider:

[![Patreon](https://img.shields.io/badge/Patreon-F96854?style=for-the-badge&logo=patreon&logoColor=white)](https://www.patreon.com/c/PegasusHeavyIndustries)

---

<div align="center">

**Built with ❤️ by [Pegasus Heavy Industries](https://github.com/PegasusHeavyIndustries)**

*"The future you see is the future you create."*

</div>
