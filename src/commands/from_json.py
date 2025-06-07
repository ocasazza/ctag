#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
From JSON command module for the ctag CLI tool.

This module defines the 'from_json' command for executing commands from a JSON file.
"""

from typing import Dict, List

import click

from src.cql import CQLProcessor
from src.interactive import InteractiveHandler
from src.json_processor import (
    JSONCommand,
    read_commands_from_json,
    validate_json_file,
)
from src.models.search_results import SearchResultItem
from src.tags import TagManager
from src.utils import sanitize_text


@click.command()
@click.argument(
    "json_file",
    type=click.Path(exists=True, file_okay=True, dir_okay=False, readable=True),
)
@click.option(
    "--abort-key",
    default="q",
    help="Key to abort all operations in interactive mode",
)
@click.pass_context
def from_json(ctx, json_file, abort_key):
    """
    Execute commands from a JSON file.

    JSON_FILE is the path to a JSON file containing commands to execute.

    The JSON file should have the following structure:
    {
        "description": "Optional description of the commands",
        "commands": [
            {
                "action": "add",
                "cql_expression": "space = DOCS",
                "tags": ["tag1", "tag2"],
                "interactive": false,
                "cql_exclude": null
            },
            {
                "action": "replace",
                "cql_expression": "space = DOCS",
                "tags": {
                    "old1": "new1",
                    "old2": "new2"
                },
                "interactive": true,
                "cql_exclude": "space = ARCHIVE"
            }
        ]
    }

    Example:
        ctag from_json examples/commands.json
    """
    confluence = ctx.obj["CONFLUENCE"]
    dry_run = ctx.obj["DRY_RUN"]

    # Validate the JSON file
    if not validate_json_file(json_file):
        click.echo(f"Error: Invalid JSON file format: {json_file}")
        return

    # Read commands from the JSON file
    try:
        commands = read_commands_from_json(json_file)
    except Exception as e:
        click.echo(f"Error reading JSON file: {str(e)}")
        return

    if not commands:
        click.echo("No valid commands found in the JSON file.")
        return

    click.echo(f"Found {len(commands)} commands in the JSON file.")

    # Initialize our processors
    cql_processor = CQLProcessor(confluence)
    tag_manager = TagManager(confluence)

    # Process each command
    total_results = {
        "total": 0,
        "processed": 0,
        "skipped": 0,
        "success": 0,
        "failed": 0,
    }

    for i, command in enumerate(commands):
        click.echo(f"\nExecuting command {i+1}/{len(commands)}: {command}")

        # Get matching pages
        click.echo(f"Finding pages matching: {command.cql_expression}")
        pages = cql_processor.get_all_results(command.cql_expression)

        if not pages:
            click.echo("No pages found matching the CQL expression.")
            continue

        click.echo(f"Found {len(pages)} matching pages.")

        # Apply exclusion if specified
        if command.cql_exclude:
            click.echo(f"Finding pages to exclude: {command.cql_exclude}")
            excluded_pages = cql_processor.get_all_results(command.cql_exclude)
            if excluded_pages:
                original_count = len(pages)
                pages = filter_excluded_pages(pages, excluded_pages)
                click.echo(
                    f"Excluded {
        original_count -
        len(pages)} pages. {
            len(pages)} pages remaining."
                )

        if dry_run:
            click.echo("DRY RUN: No changes will be made.")
            for page in pages:
                title = sanitize_text(page.title if page.title else "Unknown")
                space = (
                    page.resultGlobalContainer.title
                    if page.resultGlobalContainer and page.resultGlobalContainer.title
                    else "Unknown"
                )

                if command.action == "add":
                    click.echo(
                        f"Would add tags {
        command.tags} to '{title}' (Space: {space})"
                    )
                elif command.action == "remove":
                    click.echo(
                        f"Would remove tags {
        command.tags} from '{title}' (Space: {space})"
                    )
                else:  # replace
                    click.echo(
                        f"Would replace tags {
        list(
            command.tag_mapping.keys())} with {
                list(
                    command.tag_mapping.values())} on '{title}' (Space: {space})"
                    )
            continue

        # Set up interactive handler if needed
        interactive_handler = None
        if command.interactive:
            interactive_handler = InteractiveHandler(default_response=True, abort_value=abort_key)

        # Process the pages
        if command.action in ("add", "remove"):
            results = tag_manager.process_pages(
                pages=pages,
                action=command.action,
                tags=command.tags,
                interactive=command.interactive,
                interactive_handler=interactive_handler,
            )
        else:  # replace
            results = tag_manager.process_pages(
                pages=pages,
                action=command.action,
                tag_mapping=command.tag_mapping,
                interactive=command.interactive,
                interactive_handler=interactive_handler,
            )

        # Update total results
        for key in total_results:
            total_results[key] += results[key]

        # Display results for this command
        click.echo(f"\nResults for command {i+1}:")
        click.echo(f"  Total pages: {results['total']}")
        click.echo(f"  Processed: {results['processed']}")
        click.echo(f"  Skipped: {results['skipped']}")
        click.echo(f"  Successful: {results['success']}")
        click.echo(f"  Failed: {results['failed']}")

        # Check if aborted
        if results.get("aborted", False):
            click.echo("\nAborted by user. Stopping execution.")
            break

    # Display overall results
    click.echo(f"\nOverall Results:")
    click.echo(f"  Total pages: {total_results['total']}")
    click.echo(f"  Processed: {total_results['processed']}")
    click.echo(f"  Skipped: {total_results['skipped']}")
    click.echo(f"  Successful: {total_results['success']}")
    click.echo(f"  Failed: {total_results['failed']}")


def filter_excluded_pages(
    pages: List[SearchResultItem], excluded_pages: List[SearchResultItem]
) -> List[SearchResultItem]:
    """Filter out pages that are in the excluded_pages list based on page ID.

    Args:
        pages: List of pages to filter
        excluded_pages: List of pages to exclude

    Returns:
        Filtered list of pages
    """
    excluded_ids = [page.content.id for page in excluded_pages if page.content and page.content.id]
    return [page for page in pages if not (page.content and page.content.id and page.content.id in excluded_ids)]
