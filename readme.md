# Confluence Tag Management CLI

A command line tool for managing tags on Confluence pages in bulk. This tool allows you to add, remove, or replace tags on multiple pages at once, using CQL (Confluence Query Language) expressions to select the pages to operate on.

## Features

- **Bulk Tag Management**: Add, remove, or replace tags on multiple pages at once
- **CQL Filtering**: Use Confluence Query Language to select pages based on various criteria
- **Interactive Mode**: Confirm each action individually before it's executed
- **Dry Run Mode**: Preview changes without making any modifications

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
# Get help on tag commands
ctag tags --help

# Add tags to pages
ctag tags add "space = DOCS" tag1 tag2 tag3

# Remove tags from pages with interactive confirmation
ctag tags remove "title ~ 'Project*'" tag1 tag2 --interactive

# Replace tags on pages
ctag tags replace "lastmodified > -7d" old1=new1 old2=new2
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
ctag tags add "space = DOCS" tag1 tag2 --interactive
```

You can specify a custom abort key with `--abort-key`:

```sh
ctag tags add "space = DOCS" tag1 tag2 --interactive --abort-key x
```

### Dry Run Mode

Add the `--dry-run` flag to preview changes without making any modifications:

```sh
ctag tags add "space = DOCS" tag1 tag2 --dry-run
```

## Documentation

- [Architecture](docs/architecture.md): Overview of the tool's architecture and components
- [CQL Examples](docs/cql-examples.md): Examples of CQL expressions for selecting pages
