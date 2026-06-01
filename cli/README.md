# devc

CLI tool for generating development containers.

## Installation

```bash
curl -fsSL https://samitox4.github.io/devc/install.sh | bash
```

## Usage

```bash
# Generate a devcontainer
devc gen nodejs --name my-project

# List available templates
devc list

# Update templates
devc update
```

## Commands

- `devc gen <template>` - Generate a devcontainer
- `devc list` - List available templates
- `devc update` - Update templates from repository

## Templates

- nodejs
- android
- react-native
- java
- laravel
- rust
- go
- python
