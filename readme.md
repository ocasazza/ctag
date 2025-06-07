# Confluence Tag Management CLI

A command line tool for managing tags on Confluence pages in bulk. This tool allows you to add, remove, or replace tags on multiple pages at once, using CQL (Confluence Query Language) expressions to select the pages to operate on.

## Features

- **Bulk Tag Management**: Add, remove, or replace tags on multiple pages at once
- **CQL Filtering**: Use Confluence Query Language to select pages based on various criteria
- **Interactive Mode**: Confirm each action individually before it's executed
- **Dry Run Mode**: Preview changes without making any modifications
- **Command Files**: Execute multiple tag operations from JSON files
- **Stdin Support**: Pipe JSON data directly to the tool

## Installation

### Using Nix Flakes (Recommended)

If you have Nix with flakes enabled:

```sh
# Clone the repository
git clone https://github.com/ocasazza/ctag.git
cd ctag

# Enter development shell
nix develop

# Or run directly without installing
nix run . -- --help
```

For automatic environment setup with direnv:

```sh
# Install direnv if not already installed
# Then allow the .envrc file
direnv allow
```

### From Source (Traditional)

```sh
git clone https://github.com/ocasazza/ctag.git
cd ctag
pip install -e .
```

### Nix Flake Features

The Nix flake provides:

- **Development Shell**: Complete development environment with all dependencies
- **Package Build**: Build the ctag package with `nix build`
- **Direct Execution**: Run ctag without installation using `nix run`
- **Reproducible Environment**: Consistent development environment across machines

Development shell includes:
- Python 3 with all runtime dependencies
- Development tools (pytest, flake8, black, isort, mypy)
- Automatic PYTHONPATH setup
- Auto-creation of .env file from template

Available nix commands:
```sh
nix develop          # Enter development shell
nix build            # Build the package
nix run . -- --help  # Run ctag directly
nix flake check      # Validate the flake
```

## Configuration

The tool requires authentication with your Confluence instance. You can configure it using environment variables or a `.env` file:

### .env File

Create a `.env` file in your current directory:

```
ATLASSIAN_URL=https://your-instance.atlassian.net
ATLASSIAN_USERNAME=your-email@example.com
ATLASSIAN_TOKEN=your-api-token
```

To generate Atlassian tokens, see Atlassian's documentation for managing API keys: [Manage API Tokens For Your Atlassian Account](https://support.atlassian.com/atlassian-account/docs/manage-api-tokens-for-your-atlassian-account/)

## Usage

### General Help

```sh
ctag --help
```

### Tag Management Commands

```sh
# Add tags to pages
ctag add "space = DOCS" tag1 tag2 tag3

# Remove tags from pages with interactive confirmation
ctag remove "title ~ 'Project*'" tag1 tag2 --interactive

# Replace tags on pages
ctag replace "lastmodified > -7d" old1=new1 old2=new2
```

### Using CQL Expressions

CQL (Confluence Query Language) expressions allow you to select pages based on various criteria. Some examples:

```
# Pages in a specific space
space = "DOCS"

# Pages with a specific word in the title
title ~ "Project"

# Pages modified in the last 7 days
lastmodified > "-7d"

# Pages with a specific tag
label = "documentation"

# Combining conditions
space = "DOCS" AND lastmodified > "-7d"
```

For more examples and details on CQL syntax, see the [CQL Examples](docs/cql-examples.md) documentation.

### Interactive Mode

Add the `--interactive` flag to any command to confirm each action before it's executed:

```sh
ctag add "space = DOCS" tag1 tag2 --interactive
```

You can specify a custom abort key with `--abort-key`:

```sh
ctag add "space = DOCS" tag1 tag2 --interactive --abort-key x
```

### Dry Run Mode

Add the `--dry-run` flag to preview changes without making any modifications:

```sh
ctag add "space = DOCS" tag1 tag2 --dry-run
```

### Using Command Files and Input Methods

#### JSON Command Files

You can execute multiple tag operations from a JSON file:

```sh
ctag from_json examples/commands.json
```

The JSON file should have the following structure:

```json
{
  "description": "Optional description of the commands",
  "commands": [
    {
      "action": "add",
      "cql_expression": "space = DOCS AND title ~ 'Project'",
      "tags": ["documentation", "project"],
      "interactive": false,
      "cql_exclude": null
    },
    {
      "action": "replace",
      "cql_expression": "space = DOCS AND label = 'old-tag'",
      "tags": {
        "old-tag": "new-tag",
        "typo": "correct"
      },
      "interactive": false,
      "cql_exclude": "label = 'do-not-modify'"
    }
  ]
}
```

The JSON file is validated against a schema to ensure it has the correct format. Each command in the file is executed sequentially.

For `add` and `remove` actions, the `tags` field should be an array of strings. For `replace` actions, the `tags` field should be an object mapping old tags to new tags.

A JSON schema file is provided in `examples/schema.json` that you can use to validate your JSON files or set up autocompletion in your editor.

#### Using Stdin (Pipes)

You can pipe JSON data directly to the tool:

```sh
# Pipe JSON data
cat examples/commands.json | ctag from_stdin_json

# Generate JSON data dynamically and pipe it
echo '{"commands":[{"action":"add","cql_expression":"space = DOCS","tags":["tag1"]}]}' | ctag from_stdin_json
```

#### Dry Run Mode

You can combine any of these input methods with the `--dry-run` flag to preview the changes:

```sh
ctag from_json examples/commands.json --dry-run
ctag from_stdin_json --dry-run < examples/commands.json
```

## Documentation

- [Architecture](docs/architecture.md): Overview of the tool's architecture and components
- [CQL Examples](docs/cql-examples.md): Examples of CQL expressions for selecting pages
