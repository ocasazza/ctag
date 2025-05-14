# Confluence Tag Management CLI Architecture

This document outlines the architecture of the Confluence Tag Management CLI (ctag).

## Overview

The CLI provides functionality for managing tags on Confluence pages in bulk, using CQL (Confluence Query Language) expressions to select pages and perform tag operations. The tool supports adding, removing, and replacing tags, with an interactive mode for confirming each action.

## Components

### 1. Main CLI Entry Point (`src/main.py`)

The main entry point for the application, which:
- Handles command-line arguments and options
- Initializes the Confluence client
- Provides the CLI interface with commands for tag management
- Coordinates the interaction between different components

### 2. Tag Manager (`src/tags.py`)

Responsible for tag operations on Confluence pages:
- Adding tags to pages
- Removing tags from pages
- Replacing tags on pages
- Processing multiple pages with tag operations

### 3. CQL Processor (`src/cql.py`)

Handles CQL queries to find matching pages:
- Executing CQL queries against the Confluence API
- Retrieving page information
- Handling pagination for large result sets

### 4. Interactive Handler (`src/interactive.py`)

Manages interactive confirmations for operations:
- Prompting the user for confirmation before each action
- Handling user responses (yes/no)
- Supporting an abort mechanism to stop all remaining operations

## Command Structure

```
ctag
└── tags
    ├── add <cql_expression> <tags...> [--interactive] [--abort-key KEY]
    ├── remove <cql_expression> <tags...> [--interactive] [--abort-key KEY]
    └── replace <cql_expression> <tag_pairs...> [--interactive] [--abort-key KEY]
```

## Data Flow

1. User enters a command with a CQL expression and tag parameters
2. The CLI parses the command and options
3. The CQL processor executes the query to find matching pages
4. For each matching page:
   - If in interactive mode, the user is prompted for confirmation
   - The tag manager performs the requested operation on the page
5. Results are displayed to the user

## Configuration

The tool is configured using environment variables or a `.env` file:
- `CONFLUENCE_URL`: The base URL of the Confluence instance
- `CONFLUENCE_USERNAME`: The username for authentication
- `ATLASSIAN_TOKEN`: The API token for authentication

## Example Usage

```bash
# Add tags to all pages in the DOCS space
ctag tags add "space = DOCS" tag1 tag2 tag3

# Remove tags from pages with specific title, with interactive confirmation
ctag tags remove "title ~ 'Project*'" tag1 tag2 --interactive

# Replace tags on pages modified in the last week
ctag tags replace "lastmodified > -7d" old1=new1 old2=new2 --interactive
