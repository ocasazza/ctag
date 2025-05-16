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
- Retrieving page information as `SearchResultItem` objects
- Handling pagination for large result sets

### 4. Interactive Handler (`src/interactive.py`)

Manages interactive confirmations for operations:
- Prompting the user for confirmation before each action
- Handling user responses (yes/no)
- Supporting an abort mechanism to stop all remaining operations

### 5. JSON Processor (`src/json_processor.py`)

Handles reading and validating JSON command files:
- Reading commands from JSON files
- Validating JSON against a schema
- Converting JSON data to command objects

### 6. CSV Processor (`src/csv_processor.py`)

Handles reading and validating CSV command files:
- Reading commands from CSV files
- Validating CSV file structure
- Parsing CSV data into command objects

### 7. Stdin Processor (`src/stdin_processor.py`)

Handles reading commands from stdin and command line arguments:
- Reading JSON and CSV data from stdin (pipes)
- Validating the input data structure
- Converting input data to command objects

## Command Structure

```
ctag
├── add <cql_expression> <tags...> [--interactive] [--abort-key KEY] [--cql-exclude CQL]
├── remove <cql_expression> <tags...> [--interactive] [--abort-key KEY] [--cql-exclude CQL]
├── replace <cql_expression> <tag_pairs...> [--interactive] [--abort-key KEY] [--cql-exclude CQL]
├── from_json <json_file> [--abort-key KEY]
├── from_csv <csv_file> [--abort-key KEY]
├── from_stdin_json [--abort-key KEY]
├── from_stdin_csv [--abort-key KEY]
├── from_json_string <json_data> [--abort-key KEY]
└── from_csv_string <csv_data> [--abort-key KEY]
```

## Data Flow

### Standard Command Flow

1. User enters a command with a CQL expression and tag parameters
2. The CLI parses the command and options
3. The CQL processor executes the query to find matching pages
4. For each matching page:
   - If in interactive mode, the user is prompted for confirmation
   - The tag manager performs the requested operation on the page
5. Results are displayed to the user

### JSON Command File Flow

1. User enters the `from_json` command with a path to a JSON file
2. The CLI parses the command and options
3. The JSON processor reads and validates the JSON file
4. For each command in the JSON file:
   - The CQL processor executes the query to find matching pages
   - If a CQL exclude expression is provided, matching pages are filtered out
   - For each matching page:
     - If interactive mode is enabled for the command, the user is prompted for confirmation
     - The tag manager performs the requested operation on the page
   - Results for the command are displayed to the user
5. Overall summary is displayed to the user

### CSV Command File Flow

1. User enters the `from_csv` command with a path to a CSV file
2. The CLI parses the command and options
3. The CSV processor validates the CSV file structure and reads the commands
4. For each command in the CSV file:
   - The CQL processor executes the query to find matching pages
   - If a CQL exclude expression is provided, matching pages are filtered out
   - For each matching page:
     - If interactive mode is enabled for the command, the user is prompted for confirmation
     - The tag manager performs the requested operation on the page
   - Results for the command are displayed to the user
5. Overall summary is displayed to the user

### Stdin JSON Flow

1. User pipes JSON data to the `from_stdin_json` command
2. The CLI parses the command and options
3. The stdin processor checks if stdin has data and reads the JSON data
4. The JSON data is validated and parsed into command objects
5. For each command:
   - The CQL processor executes the query to find matching pages
   - If a CQL exclude expression is provided, matching pages are filtered out
   - For each matching page:
     - If interactive mode is enabled for the command, the user is prompted for confirmation
     - The tag manager performs the requested operation on the page
   - Results for the command are displayed to the user
6. Overall summary is displayed to the user

### Stdin CSV Flow

1. User pipes CSV data to the `from_stdin_csv` command
2. The CLI parses the command and options
3. The stdin processor checks if stdin has data and reads the CSV data
4. The CSV data is validated and parsed into command objects
5. For each command:
   - The CQL processor executes the query to find matching pages
   - If a CQL exclude expression is provided, matching pages are filtered out
   - For each matching page:
     - If interactive mode is enabled for the command, the user is prompted for confirmation
     - The tag manager performs the requested operation on the page
   - Results for the command are displayed to the user
6. Overall summary is displayed to the user

### Command Line JSON String Flow

1. User enters the `from_json_string` command with a JSON string
2. The CLI parses the command and options
3. The JSON string is parsed and validated
4. For each command in the JSON data:
   - The CQL processor executes the query to find matching pages
   - If a CQL exclude expression is provided, matching pages are filtered out
   - For each matching page:
     - If interactive mode is enabled for the command, the user is prompted for confirmation
     - The tag manager performs the requested operation on the page
   - Results for the command are displayed to the user
5. Overall summary is displayed to the user

### Command Line CSV String Flow

1. User enters the `from_csv_string` command with a CSV string
2. The CLI parses the command and options
3. The CSV string is parsed and validated
4. For each command in the CSV data:
   - The CQL processor executes the query to find matching pages
   - If a CQL exclude expression is provided, matching pages are filtered out
   - For each matching page:
     - If interactive mode is enabled for the command, the user is prompted for confirmation
     - The tag manager performs the requested operation on the page
   - Results for the command are displayed to the user
5. Overall summary is displayed to the user

## Configuration

The tool is configured using environment variables or a `.env` file:
- `CONFLUENCE_URL`: The base URL of the Confluence instance
- `CONFLUENCE_USERNAME`: The username for authentication
- `ATLASSIAN_TOKEN`: The API token for authentication

## Example Usage

```bash
# Add tags to all pages in the DOCS space
ctag add "space = DOCS" tag1 tag2 tag3

# Remove tags from pages with specific title, with interactive confirmation
ctag remove "title ~ 'Project*'" tag1 tag2 --interactive

# Replace tags on pages modified in the last week
ctag replace "lastmodified > -7d" old1=new1 old2=new2 --interactive

# Execute commands from a JSON file
ctag from_json examples/commands.json

# Execute commands from a JSON file with dry run mode
ctag from_json examples/commands.json --dry-run

# Execute commands from a CSV file
ctag from_csv examples/commands.csv

# Execute commands from a CSV file with dry run mode
ctag from_csv examples/commands.csv --dry-run

# Pipe JSON data to the tool
cat examples/commands.json | ctag from_stdin_json

# Pipe CSV data to the tool
cat examples/commands.csv | ctag from_stdin_csv

# Provide JSON data as a command line argument
ctag from_json_string '{"commands":[{"action":"add","cql_expression":"space = DOCS","tags":["tag1"]}]}'

# Provide CSV data as a command line argument
ctag from_csv_string 'action,cql_expression,tags
add,space = DOCS,tag1,tag2'
