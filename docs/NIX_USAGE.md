# Using ctag with Nix

This document explains how to use the Nix flake for the ctag project.

## Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled
- Optional: [direnv](https://direnv.net/) for automatic environment activation

## Quick Start

### Development Environment with Pure Nix

```bash
# Clone the repository
git clone https://github.com/ocasazza/ctag.git
cd ctag

# Enter the development shell (loads all packages via Nix)
nix develop

# This will:
# - Load all Python dependencies directly from Nix
# - Set up PYTHONPATH to include the src/ directory
# - Install ctag in development mode
# - Make ctag available in your shell
# - No virtual environment needed!

# ctag is now available
ctag --help
ctag add "space = DOCS" tag1 tag2 --dry-run
```

### Using direnv (recommended for regular development)

```bash
# Install direnv if not already installed
# Then allow the .envrc file
direnv allow

# The environment will be automatically activated when you cd into the directory
# - Loads all Python packages via Nix
# - Sets up PYTHONPATH automatically
# - ctag will be available immediately
ctag --help
```

### Other Options

#### Running ctag directly (no shell needed)

```bash
# Run ctag directly without entering a shell
nix run . -- --help
nix run . -- add "space = DOCS" tag1 tag2 --dry-run
```

#### Building the package

```bash
# Build the package
nix build

# The built package will be available in ./result/
./result/bin/ctag --help
```

## What the Nix Flake Provides

### Packages
- `packages.default` / `packages.ctag`: The ctag CLI tool
- Built with all runtime dependencies included
- Uses modern `pyproject.toml` configuration

### Development Shell
- Python 3.12 with all runtime and development dependencies
- Development tools: pytest, pytest-cov, pytest-mock, flake8, black, isort, mypy
- Automatic PYTHONPATH setup pointing to `src/`
- Auto-creation of `.env` file from `.env.example` if it doesn't exist
- Built-in `ctag` command available immediately

### Apps
- `apps.default` / `apps.ctag`: Direct execution of ctag
- Use with `nix run . -- <args>`

## Runtime Dependencies Included

- atlassian-python-api >= 3.32.0
- click >= 8.1.3
- tqdm >= 4.64.1
- python-dotenv >= 1.0.0
- jsonschema >= 4.17.3
- pydantic >= 2.0.0

## Development Dependencies Included

- pytest >= 7.0.0
- pytest-cov >= 4.0.0
- pytest-mock >= 3.10.0
- flake8
- black
- isort
- mypy

## Configuration

The development shell automatically:
1. Sets up the Python path to include the `src/` directory
2. Creates a `.env` file from `.env.example` if it doesn't exist
3. Provides helpful command suggestions

You'll still need to edit the `.env` file with your Confluence credentials:

```
CONFLUENCE_URL=https://your-instance.atlassian.net
CONFLUENCE_USERNAME=your-email@example.com
ATLASSIAN_TOKEN=your-api-token
```

## Common Workflows

### Development Workflow

```bash
# Enter development environment
nix develop  # or use direnv

# Run tests
python -m pytest

# Lint code
python -m flake8 src/

# Format code
python -m black src/
python -m isort src/

# Test the CLI
ctag --help
ctag add "space = TEST" tag1 --dry-run
```

### CI/CD Integration

```bash
# Check flake validity
nix flake check

# Build package
nix build

# Run tests in clean environment
nix develop --command python -m pytest
```

### Installing for System Use

```bash
# Install to user profile
nix profile install .

# Or add to your system configuration if using NixOS
```

## Troubleshooting

### Git Warnings
You may see warnings about uncommitted changes. This is normal during development and doesn't affect functionality.

### Missing JSON Schema Files
The flake automatically includes the required JSON schema files (`src/models/*.json`) in the built package.

### Environment Variables
Make sure your `.env` file is properly configured with valid Confluence credentials before running ctag commands that interact with Confluence.
