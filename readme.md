# Confluence Tag Management CLI

A command line tool for managing tags on Confluence pages in bulk. This tool allows you to add, remove, or replace tags on multiple pages at once, using CQL (Confluence Query Language) expressions to select the pages to operate on.

## Features

- **Bulk Tag Management**: Add, remove, or replace tags on multiple pages at once
- **CQL Filtering**: Use Confluence Query Language to select pages based on various criteria
- **Interactive Mode**: Confirm each action individually before it's executed
- **Dry Run Mode**: Preview changes without making any modifications
- **Command Files**: Execute multiple tag operations from JSON or CSV files
- **Stdin Support**: Pipe JSON or CSV data directly to the tool
- **Command Line Arguments**: Provide JSON or CSV data directly as command line arguments

## Installation

### From Source

```sh
git clone https://github.com/ocasazza/ctag.git
cd ctag
pip install -e .
```

## Configuration

The tool requires authentication with your Confluence instance. You can configure it using environment variables or a `.env` file:

### .env File

Create a `.env` file in your current directory:

```
CONFLUENCE_URL=https://your-instance.atlassian.net
CONFLUENCE_USERNAME=your-email@example.com
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

#### CSV Command Files

You can also execute multiple tag operations from a CSV file:

```sh
ctag from_csv examples/commands.csv
```

The CSV file should have the following columns:
- `action`: The action to perform (add, remove, or replace)
- `cql_expression`: The CQL query to select pages
- `tags`: For add/remove actions, a comma-separated list of tags; for replace actions, a comma-separated list of old=new pairs
- `interactive`: (optional) Whether to confirm each action interactively (true/false)
- `cql_exclude`: (optional) CQL expression to match pages that should be excluded

Example CSV file:
```csv
action,cql_expression,tags,interactive,cql_exclude
add,"space = DOCS AND title ~ ""Project""","documentation,project",false,
remove,space = ARCHIVE,"outdated,deprecated",true,"label = ""keep"""
replace,"space = DOCS AND label = ""old-tag""","old-tag=new-tag,typo=correct",false,"label = ""do-not-modify"""
```

#### Using Stdin (Pipes)

You can pipe JSON or CSV data directly to the tool:

```sh
# Pipe JSON data
cat examples/commands.json | ctag from_stdin_json

# Generate JSON data dynamically and pipe it
echo '{"commands":[{"action":"add","cql_expression":"space = DOCS","tags":["tag1"]}]}' | ctag from_stdin_json

# Pipe CSV data
cat examples/commands.csv | ctag from_stdin_csv

# Generate CSV data dynamically and pipe it
echo 'action,cql_expression,tags
add,space = DOCS,tag1,tag2' | ctag from_stdin_csv
```

#### Using Command Line Arguments

You can also provide JSON or CSV data directly as command line arguments:

```sh
# Provide JSON data as a command line argument
ctag from_json_string '{"commands":[{"action":"add","cql_expression":"space = DOCS","tags":["tag1"]}]}'

# Provide CSV data as a command line argument
ctag from_csv_string 'action,cql_expression,tags
add,space = DOCS,tag1,tag2'
```

#### Dry Run Mode

You can combine any of these input methods with the `--dry-run` flag to preview the changes:

```sh
ctag from_json examples/commands.json --dry-run
ctag from_csv examples/commands.csv --dry-run
ctag from_stdin_json --dry-run < examples/commands.json
ctag from_json_string '{"commands":[...]}' --dry-run
```

## Documentation

- [Architecture](docs/architecture.md): Overview of the tool's architecture and components
- [CQL Examples](docs/cql-examples.md): Examples of CQL expressions for selecting pages
