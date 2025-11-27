# ctag - Confluence Tag Manager

A command-line tool for managing tags on Confluence pages in bulk, written in Rust with Nix for environment management.

## Features

- **Add, remove, or replace tags** on Confluence pages in bulk
- **Use CQL queries** to select pages based on various criteria
- **Interactive mode** to confirm each action before execution
- **Dry-run mode** to preview changes without making modifications
- **Progress bars** for long-running operations
- **JSON-based batch operations** for complex workflows
- **Multiple output formats** (table, JSON) for the `get` command

## Prerequisites

- Nix with flakes enabled
- Confluence Cloud instance with API access

## Setup

### 1. Enter the development environment

```bash
nix develop
```

This will set up all required dependencies including Rust, Cargo, and development tools.

### 2. Build the project

```bash
cargo build --release
```

### 3. Configure environment variables

Create a `.env` file in the project root with your Confluence credentials:

```env
ATLASSIAN_URL=https://your-domain.atlassian.net
ATLASSIAN_USERNAME=your-email@example.com
ATLASSIAN_TOKEN=your-api-token
```

To generate an API token, visit: https://id.atlassian.com/manage-profile/security/api-tokens

## Usage

### Basic Commands

#### Add tags to pages

```bash
ctag add "space = DOCS" tag1 tag2 tag3
```

#### Remove tags from pages

```bash
ctag remove "space = DOCS" old-tag
```

#### Replace tags

```bash
ctag replace "space = DOCS" old-tag=new-tag another-old=another-new
```

#### Get tags from pages

```bash
# Show all pages with their tags
ctag get "space = DOCS"

# Show only unique tags
ctag get "space = DOCS" --tags-only

# Output as JSON
ctag get "space = DOCS" --format json

# Save to file
ctag get "space = DOCS" --output-file results.json
```

### Advanced Options

#### Interactive mode

Confirm each action before execution:

```bash
ctag add "space = DOCS" new-tag --interactive
```

#### Dry run

Preview changes without making modifications:

```bash
ctag --dry-run add "space = DOCS" new-tag
```

#### Exclude pages

Exclude specific pages from the operation:

```bash
ctag add "space = DOCS" new-tag --cql-exclude "title ~ 'Archive*'"
```

### Batch Operations

#### From JSON file

Create a JSON file with multiple commands:

```json
{
  "description": "Quarterly tag updates",
  "commands": [
    {
      "action": "add",
      "cql_expression": "space = DOCS AND lastmodified > -30d",
      "tags": ["recent", "q4-2024"],
      "interactive": false
    },
    {
      "action": "replace",
      "cql_expression": "space = DOCS",
      "tag_mapping": {
        "old-tag": "new-tag",
        "deprecated": "archived"
      },
      "interactive": true,
      "cql_exclude": "title ~ 'Template*'"
    }
  ]
}
```

Execute the commands:

```bash
ctag from-json commands.json
```

#### From stdin

```bash
cat commands.json | ctag from-stdin-json
```

Or:

```bash
echo '{"commands":[{"action":"add","cql_expression":"space = DOCS","tags":["test"]}]}' | ctag from-stdin-json
```

## CQL Query Examples

See [docs/cql-examples.md](docs/cql-examples.md) for more CQL query examples.

## Development

### Run tests

```bash
cargo test
```

### Format code

```bash
cargo fmt
```

### Lint code

```bash
cargo clippy
```

### Using just

The project includes a `justfile` for common tasks:

```bash
just build    # Build the project
just run      # Run the project
just test     # Run tests
just fmt      # Format code
just check    # Run clippy
```

## Building with Nix

To build the project using Nix:

```bash
nix build
```

The binary will be available in `./result/bin/ctag`.

## Documentation

Documentation is auto-generated from the source code. To view it:

```bash
cargo doc --open
```

## License

See [LICENSE](LICENSE) file for details.

## Migration from Python

This project was migrated from Python to Rust. The CLI interface remains exactly the same, so all existing scripts and workflows should continue to work without modification.
