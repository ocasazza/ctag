# Architecture

## Project Description
This repository contains code for ctag, a Python CLI tool used for managing Confluence page tags in bulk. It uses CQL (Confluence Query Language) queries to select pages and performs add/remove/replace tag operations.

## Instructions

Read the following items to learn more about the current project before starting a task. If any changes result in inaccurate information in this file during the completion of a task, please update this file to reflect the correct information.

## CLI Structure

### Main CLI Entry Point
```
ctag [GLOBAL_OPTIONS] COMMAND [COMMAND_OPTIONS] [ARGUMENTS]
```

### Global Options (Available for all commands)
- `--version` - Show version and exit
- `--progress BOOLEAN` - Show progress bars during operations (default: True)
- `--recurse BOOLEAN` - Process child pages recursively (default: True)
- `--dry-run` - Preview changes without making any modifications

### Commands

#### 1. `add` - Add tags to pages
```
ctag add [OPTIONS] CQL_EXPRESSION TAGS...
```
**Arguments:**
- `CQL_EXPRESSION` - Confluence Query Language expression to select pages
- `TAGS...` - One or more tags to add (space-separated)

**Options:**
- `--interactive` - Confirm each action interactively
- `--abort-key TEXT` - Key to abort all operations in interactive mode (default: 'q')
- `--cql-exclude TEXT` - CQL expression to match pages that should be excluded

**Example:** `ctag add "space = DOCS" tag1 tag2 tag3`

#### 2. `remove` - Remove tags from pages
```
ctag remove [OPTIONS] CQL_EXPRESSION TAGS...
```
**Arguments:**
- `CQL_EXPRESSION` - Confluence Query Language expression to select pages
- `TAGS...` - One or more tags to remove (space-separated)

**Options:**
- `--interactive` - Confirm each action interactively
- `--abort-key TEXT` - Key to abort all operations in interactive mode (default: 'q')
- `--cql-exclude TEXT` - CQL expression to match pages that should be excluded

**Example:** `ctag remove "space = DOCS" tag1 tag2`

#### 3. `replace` - Replace tags on pages
```
ctag replace [OPTIONS] CQL_EXPRESSION TAG_PAIRS...
```
**Arguments:**
- `CQL_EXPRESSION` - Confluence Query Language expression to select pages
- `TAG_PAIRS...` - One or more old=new tag pairs (space-separated)

**Options:**
- `--interactive` - Confirm each action interactively
- `--abort-key TEXT` - Key to abort all operations in interactive mode (default: 'q')
- `--cql-exclude TEXT` - CQL expression to match pages that should be excluded

**Example:** `ctag replace "space = DOCS" old1=new1 old2=new2`

#### 4. `from-json` - Execute commands from JSON file
```
ctag from-json [OPTIONS] JSON_FILE
```
**Arguments:**
- `JSON_FILE` - Path to JSON file containing commands

**Options:**
- `--abort-key TEXT` - Key to abort all operations in interactive mode (default: 'q')

**JSON Structure:**
```json
{
  "description": "Optional description",
  "commands": [
    {
      "action": "add|remove|replace",
      "cql_expression": "CQL query",
      "tags": ["tag1", "tag2"] | {"old": "new"},
      "interactive": false,
      "cql_exclude": "optional CQL exclude"
    }
  ]
}
```

#### 5. `get` - Get tags from pages
```
ctag get [OPTIONS] CQL_EXPRESSION
```
**Arguments:**
- `CQL_EXPRESSION` - Confluence Query Language expression to select pages

**Options:**
- `--format CHOICE` - Output format: table, json (default: table)
- `--show-pages/--no-show-pages` - Include page titles and spaces in output (default: True)
- `--tags-only` - Show only unique tags across all pages
- `--interactive` - Browse results interactively
- `--abort-key TEXT` - Key to abort all operations in interactive mode (default: 'q')
- `--cql-exclude TEXT` - CQL expression to match pages that should be excluded
- `--output-file PATH` - Save results to file

**Examples:**
- `ctag get "space = DOCS"` - Get all tags from pages in DOCS space
- `ctag get "space = DOCS" --tags-only` - Show only unique tags
- `ctag get "lastmodified > -7d" --format json` - Export recent pages' tags as JSON
- `ctag get "title ~ 'Project*'" --interactive` - Browse results interactively

#### 6. `from-stdin-json` - Execute commands from stdin JSON
```
ctag from-stdin-json [OPTIONS]
```
**Options:**
- `--abort-key TEXT` - Key to abort all operations in interactive mode (default: 'q')

**Usage:** `cat commands.json | ctag from-stdin-json`

## Key Components

### Core Modules
- `src/main.py` - CLI entry point with Click framework, handles authentication and global options
- `src/commands/` - Individual command implementations (add, remove, replace, from_json, from_stdin_json)
- `src/tags.py` - TagManager class for performing tag operations on Confluence pages
- `src/cql.py` - CQLProcessor class for executing Confluence queries and handling pagination
- `src/models/` - Pydantic models generated from JSON schemas for data validation

### Supporting Modules
- `src/interactive.py` - InteractiveHandler for user confirmations
- `src/json_processor.py` - JSON command file processing and validation
- `src/stdin_processor.py` - Stdin input processing for piped data
- `src/utils/` - Utility functions for text processing and Pydantic model generation

## Development Rules

### Code Organization
- Follow the existing modular structure when adding new features
- New CLI commands go in `src/commands/` following the established pattern
- Use Pydantic models for data validation (generated from JSON schemas in `src/models/`)
- Maintain separation of concerns: CQL processing, tag management, and user interaction are separate

### Testing
- Use pytest framework for all tests
- Tests are located in `tests/` directory mirroring the source structure
- Maintain test coverage for new features

### Dependencies
- Core: `atlassian-python-api`, `click`, `pydantic`, `python-dotenv`
- Development: `pytest`, `pytest-cov`, `flake8`
- Follow existing dependency patterns when adding new requirements

### Configuration
- Environment-based configuration using `.env` file
- Required variables: `CONFLUENCE_URL`, `CONFLUENCE_USERNAME`, `ATLASSIAN_TOKEN`
- Global CLI options: `--dry-run`, `--progress`, `--recurse`

### Command Pattern
Each command follows this structure:
1. Parse CQL expression and parameters
2. Execute CQL query via CQLProcessor
3. Filter excluded pages if specified
4. Process pages with TagManager
5. Handle interactive confirmations if enabled
