# CTAG Test Suite

This directory contains end-to-end tests for the Confluence Tag Management CLI (ctag).

## Test Structure

The test suite is organized by command:

- `test_add_command.py`: Tests for the `add` command
- `test_remove_command.py`: Tests for the `remove` command
- `test_replace_command.py`: Tests for the `replace` command
- `test_from_json.py`: Tests for the `from-json` command
- `test_from_csv.py`: Tests for the `from-csv` command
- `test_from_stdin_json.py`: Tests for the `from-stdin-json` command

## Test Fixtures

The test fixtures are defined in `conftest.py`:

- `confluence_client`: Creates a Confluence client for API operations
- `test_page`: Creates a test page for tag operations and cleans it up after the test
- `cleanup_tags`: Tracks and cleans up tags added during tests

## Running Tests

To run all tests:

```bash
pytest
```

To run tests for a specific command:

```bash
pytest tests/test_add_command.py
```

To run a specific test:

```bash
pytest tests/test_add_command.py::test_add_single_tag
```

To run tests with verbose output:

```bash
pytest -v
```

## Test Environment

The tests require a Confluence environment to run against. The following environment variables must be set:

- `CONFLUENCE_URL`: The base URL of the Confluence instance
- `CONFLUENCE_USERNAME`: The username for authentication
- `ATLASSIAN_TOKEN`: The API token for authentication

These can be set in a `.env` file in the project root.

## Test Design

The tests are designed to be independent and idempotent. Each test:

1. Creates a test page with a unique title
2. Performs tag operations on the page
3. Verifies the operations were successful
4. Cleans up the page and tags

The tests use direct API calls to verify the results of the commands, rather than relying on the command output.

## CQL Expressions

The tests use `contentId = {page_id}` as the CQL expression to find pages, as this is the most reliable way to find a specific page. Other CQL expressions like `space = {space_key} AND title = '{title}'` may not work reliably due to indexing delays or other issues.

## Tag Cleanup

Tags added during tests are tracked and cleaned up after the test completes. This ensures that future test runs start with a clean slate.

## Test Data

The tests generate random tag names to avoid conflicts between test runs.
