# devc

CLI tool for generating ready-to-use development containers.

## Overview

`devc` is a command-line interface that generates development container configurations for various technologies. It downloads templates from the [devcontainers](https://github.com/SamitoX4/devcontainers) repository and sets them up in your project.

## Features

- 🚀 Instant dev environment setup
- 📦 Multiple technology templates
- 🔄 Auto-updates for templates
- 💾 Offline support with local cache
- ⚙️ Git configuration support
- 🎯 Interactive mode with guided prompts

## Installation

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/SamitoX4/devc/main/pages/install.sh | bash
```

### From Source

```bash
git clone https://github.com/SamitoX4/devc.git
cd devc/cli
cargo build --release
cp target/release/devc ~/.local/bin/
```

Make sure `~/.local/bin` is in your PATH.

## Usage

### Interactive Mode (Recommended)

Just run:

```bash
devc gen
```

The CLI will prompt you for:
1. Select a template (arrow keys)
2. Project name (default: current directory)
3. Git User Name
4. Git User Email

### With Flags

Skip prompts by passing flags:

```bash
# Specify template and project name
devc gen --template nodejs --name my-project

# With Git configuration
devc gen --template react-native --name my-app --git-name "John Doe" --git-email "john@example.com"

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

| Template | Description |
|----------|-------------|
| `nodejs` | Node.js with TypeScript, npm/pnpm |
| `android` | Java 17 + Android SDK |
| `react-native` | Node.js + React Native + Android |
| `java` | Java 17 + Maven |
| `laravel` | PHP 8.3 + Composer |
| `rust` | Rust (stable) + Cargo |
| `go` | Go 1.22 |
| `python` | Python 3.12 + pip |

## Commands

```
devc gen [options]              Generate a devcontainer (interactive if no options)
devc list [options]             List available templates
devc update [options]           Update templates from repository
devc config [options]          Configure Git user settings

Options:
  -t, --template <name>        Template name (e.g., nodejs, react-native)
  -n, --name <name>            Project name
  --git-name <name>            Git user name
  --git-email <email>          Git user email
  --verbose                    Verbose output
```

## How It Works

1. **First Run**: Downloads templates to `~/.devc/cache/`
2. **Interactive Mode**: Guides you through setup with prompts
3. **Flag Mode**: Skip prompts by passing options
4. **Git Configuration**: Saved to `~/.devc/config.json` for future use
5. **Each Execution**: Checks for updates in the background
6. **Offline Mode**: Uses cached templates when offline

## Project Structure

```
devc/
├── cli/               # Rust CLI source code
│   ├── src/
│   │   ├── commands/  # CLI commands (gen, list, update, config)
│   │   └── utils/     # Cache, fetcher, copier, merger
│   └── Cargo.toml
├── pages/             # GitHub Pages (landing + install)
│   ├── index.html
│   └── install.sh
└── scripts/          # Build scripts
    └── build-release.sh
```

## Creating a Release

To create a GitHub release with precompiled binaries:

```bash
cd scripts
chmod +x build-release.sh
./build-release.sh
```

This will create a release package. Then:

1. Go to https://github.com/SamitoX4/devc/releases/new
2. Create a new tag (e.g., v0.1.0)
3. Upload the `.tar.gz` files for each platform
4. Publish the release

After the release is published, users can install using:
```bash
curl -fsSL https://raw.githubusercontent.com/SamitoX4/devc/main/pages/install.sh | bash
```

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
