# ctag - Confluence Tag Manager

A command-line tool for managing tags on Confluence pages in bulk, written in Rust with Nix for environment management.

![ctag demo](demo.gif)

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

### Regular Expression Support

#### Remove tags by pattern

```bash
# Remove all tags starting with "test-tag-"
ctag remove "space = DOCS" "test-tag-.*" --regex
```

#### Replace tags by pattern

```bash
# Replace any tag matching "id-[0-9]+" with "matched-id"
# Note: Use positional pairs (pattern replacement pattern replacement ...)
ctag replace --regex "space = DOCS" "id-[0-9]+" "matched-id"

# Multiple replacements
ctag replace --regex "space = DOCS" \
  "test-.*" "new-test" \
  "id-[0-9]+" "matched-id"
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
      "tags": {
        "old-tag": "new-tag",
        "deprecated": "archived"
      },
      "interactive": true
    },
    {
      "action": "replace",
      "cql_expression": "space = DOCS",
      "tags": {
        "test-.*": "new-test",
        "id-[0-9]+": "matched-id"
      },
      "regex": true
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
