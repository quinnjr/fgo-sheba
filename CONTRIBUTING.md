# Contributing to FGO Sheba

First off, thank you for considering contributing to FGO Sheba! 🎉

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)

## Code of Conduct

This project and everyone participating in it is governed by our commitment to creating a welcoming and inclusive environment. Please be respectful and constructive in all interactions.

## How Can I Contribute?

### 🐛 Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates.

When creating a bug report, include:
- **Device information** (model, Android version)
- **FGO version** (JP/NA/KR/TW and version number)
- **Steps to reproduce** the behavior
- **Expected behavior** vs actual behavior
- **Screenshots/logs** if applicable

### 💡 Suggesting Features

Feature suggestions are welcome! Please include:
- **Clear description** of the feature
- **Use case** - why would this be useful?
- **Possible implementation** approach (optional)

### 🔧 Code Contributions

Areas where we especially welcome contributions:

- **ML Model Improvements** - Better card/servant recognition accuracy
- **New Language Translations** - Help us reach more users
- **Battle AI Strategies** - Smarter decision-making algorithms
- **UI/UX Improvements** - Better user experience
- **Documentation** - Help others understand the project

## Development Setup

### Prerequisites

1. **Rust** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add aarch64-linux-android armv7-linux-androideabi
   ```

2. **Android NDK**
   - Install via Android Studio SDK Manager
   - Set `ANDROID_NDK_HOME` environment variable

3. **Android Studio** (or Gradle CLI)
   - JDK 17+
   - Gradle 8.5+

### Building

```bash
# Clone and enter directory
git clone https://github.com/quinnjr/fgo-sheba.git
cd fgo-sheba

# Build Rust library
cargo build --release

# Build Android app
cd android
./gradlew assembleDebug
```

### Testing

```bash
# Rust tests
cargo test

# Android tests
cd android && ./gradlew test
```

## Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch from `main`
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make** your changes
4. **Test** your changes thoroughly
5. **Commit** with clear, descriptive messages
   ```bash
   git commit -m "feat: add support for X"
   ```
6. **Push** to your fork
   ```bash
   git push origin feature/your-feature-name
   ```
7. **Open** a Pull Request

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `test:` - Adding/updating tests
- `chore:` - Maintenance tasks

## Style Guidelines

### Rust

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `cargo fmt` before committing
- Use `cargo clippy` and address warnings
- Document public APIs with rustdoc comments

### Kotlin

- Follow [Kotlin Coding Conventions](https://kotlinlang.org/docs/coding-conventions.html)
- Use meaningful variable/function names
- Add KDoc comments for public functions

### XML (Android Resources)

- Use descriptive resource names with prefixes:
  - `bg_` for backgrounds
  - `ic_` for icons
  - `btn_` for buttons
  - `tv_` for TextViews

### Translations

When adding or updating translations:
- Keep the same string keys as `values/strings.xml`
- Maintain proper grammar and natural phrasing
- Test on a device with that locale

## Questions?

Feel free to open an issue with the `question` label or reach out to the maintainers.

---

Thank you for contributing! 🙏
