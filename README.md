# devc

CLI tool for generating ready-to-use development containers.

## Overview

`devc` is a command-line interface that generates development container configurations for various technologies. It downloads templates from the [devcontainers](https://github.com/SamitoX4/devcontainers) repository and sets them up in your project.

## Features

- 🚀 Instant dev environment setup
- 📦 Multiple technology templates (including nested categories like `android/kotlin-native`)
- 🔄 Auto-updates for templates
- 💾 Offline support with local cache
- ⚙️ Git configuration support
- 🎯 Interactive mode with guided prompts
- 🎛️ **Interactive version selector** — customize tool versions (Android API, Kotlin, NDK, Node, etc.) with arrow-key selection
- 🔗 **Smart version suggestions** — Build Tools auto-suggests the matching version when you pick an Android API Level

## Installation

### npm (Recommended — works on all platforms)

```bash
npm i @blackycoderx4/devc
```

Requires Node.js 14+. The package automatically downloads the correct native binary for your platform (Windows, Linux, or macOS).

### Quick Install (Linux / macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/SamitoX4/devc/main/docs/install.sh | bash
```

### Windows

If you have Node.js installed, use the npm method above.

Alternatively:
- **WSL2**: Run the Linux installer inside your WSL2 distribution.
- **Manual**: Download the latest `.zip` for Windows from the [Releases](https://github.com/SamitoX4/devc/releases) page, extract it, and add `devc.exe` to your PATH.

### From Source

```bash
git clone https://github.com/SamitoX4/devc.git
cd devc/cli
cargo build --release
```

On Linux/macOS copy the binary to your PATH:
```bash
cp target/release/devc ~/.local/bin/
```

On Windows the binary will be at `target\release\devc.exe`.

## Usage

### Interactive Mode (Recommended)

Just run:

```bash
devc gen
```

The CLI will guide you through:
1. **Select a template** (arrow keys) — supports nested templates like `android/kotlin-native`
2. **Project name** (default: current directory)
3. **Git User Name**
4. **Git User Email**
5. **Customize versions** (optional) — if the template supports parameterized versions, you can pick from a curated list:

   ```
   KOTLIN_VERSION:
     2.0.0
     2.0.10
   > 2.0.21
     2.1.0

   ANDROID_API_LEVEL:
     33
     34
     35
   > 36

   BUILD_TOOLS_VERSION:
     33.0.0
     34.0.0
     35.0.0
   > 36.0.0   ← auto-suggested from API Level 36

   GO_VERSION:
     1.21.0
     1.22.0
     1.23.0
   > 1.24.0

   PYTHON_VERSION:
     3.11
     3.12
   > 3.13
   ```

### With Flags

Skip prompts by passing flags:

```bash
# Specify template and project name
devc gen --template nodejs --name my-project

# Nested templates (Android stack)
devc gen --template android/kotlin-native --name my-native-app
devc gen --template android/flutter --name my-flutter-app
devc gen --template android/ndk --name my-ndk-project

# With Git configuration
devc gen --template android/react-native --name my-app --git-name "John Doe" --git-email "john@example.com"

# All options
devc gen -t java -n my-java-app --git-name "John Doe" --git-email "john@example.com"
```

### Configure Git

```bash
# Interactive (prompts for name and email)
devc config

# With flags
devc config --git-name "Your Name" --git-email "your@email.com"

# Show current configuration
devc config --show
```

### List Available Templates

```bash
devc list
```

### Update Templates

```bash
devc update
```

## Available Templates

### General

| Template | Description | Customizable Versions |
|----------|-------------|----------------------|
| `nodejs` | Node.js with TypeScript, npm/pnpm | Node version, Base image variant |
| `java` | Java 17 + Maven | Maven version |
| `laravel` | PHP 8.3 + Composer | PHP version |
| `rust` | Rust (stable) + Cargo | Rust toolchain |
| `go` | Go 1.22 | Go version |
| `python` | Python 3.12 + pip | Python version |

### Android Stack

| Template | Description | Customizable Versions |
|----------|-------------|----------------------|
| `android/java` | Java 17 + Android SDK | API Level, Build Tools, NDK, CMD Line Tools |
| `android/kotlin-native` | Kotlin/Native + Android SDK | Kotlin, API Level, Build Tools, NDK, CMD Line Tools |
| `android/ndk` | Android NDK + CMake | API Level, Build Tools, NDK, CMake, CMD Line Tools |
| `android/react-native` | Node.js + React Native + Android SDK | Node version, API Level, Build Tools, NDK, CMD Line Tools |
| `android/flutter` | Flutter + Android SDK | Flutter branch, API Level, Build Tools, NDK, CMD Line Tools |

## Commands

```
devc gen [options]              Generate a devcontainer (interactive if no options)
devc list [options]             List available templates
devc update [options]           Update templates from repository
devc config [options]          Configure Git user settings

Options:
  -t, --template <name>        Template name (e.g., nodejs, android/kotlin-native)
  -n, --name <name>            Project name
  --git-name <name>            Git user name
  --git-email <email>          Git user email
  --verbose                    Verbose output
```

## How It Works

1. **First Run**: Downloads templates to `~/.devc/cache/`
2. **Template Discovery**: Automatically finds all valid templates, including nested ones like `android/kotlin-native`
3. **Interactive Mode**: Guides you through setup with prompts, including an optional version picker for parameterized templates
4. **Flag Mode**: Skip prompts by passing options
5. **Git Configuration**: Saved to `~/.devc/config.json` for future use
6. **Each Execution**: Checks for updates in the background
7. **Offline Mode**: Uses cached templates when offline

## Project Structure

```
devc/
├── .github/           # GitHub Actions workflows
│   └── workflows/
│       └── release.yml
├── cli/               # Rust CLI source code
│   ├── src/
│   │   ├── commands/  # CLI commands (gen, list, update, config)
│   │   └── utils/     # Cache, fetcher, copier, merger
│   └── Cargo.toml
├── npm/               # npm wrapper package
│   ├── bin/
│   ├── install.js
│   ├── platform.js
│   └── package.json
├── docs/              # GitHub Pages (landing + install)
│   ├── index.html
│   └── install.sh
└── scripts/          # Build scripts
    └── build-release.sh
```

## Creating a Release

Releases are built automatically with **GitHub Actions**. Simply push a version tag:

```bash
git tag v0.3.0
git push origin v0.3.0
```

GitHub Actions will compile binaries for Linux, macOS (Intel & Apple Silicon), and Windows, package them with templates, and publish them to the [Releases](https://github.com/SamitoX4/devc/releases) page.

### Manual Release (local)

If you prefer to build locally:

```bash
cd scripts
chmod +x build-release.sh
./build-release.sh
```

After the package is created, upload it manually to GitHub Releases.

## Configuration

The CLI stores configuration in:

```
~/.devc/
├── cache/
│   └── templates/     # Downloaded templates
└── config.json       # CLI configuration (Git user, etc.)
```

### config.json Example

```json
{
  "templates_version": "1.0.0",
  "last_check": "2026-03-31",
  "git": {
    "name": "Your Name",
    "email": "your@email.com"
  }
}
```

## License

MIT

## Links

- [GitHub Repository](https://github.com/SamitoX4/devc)
- [Templates Repository](https://github.com/SamitoX4/devcontainers)
