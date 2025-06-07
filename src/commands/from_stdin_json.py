#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
From stdin JSON command module for the ctag CLI tool.

This module defines the 'from_stdin_json' command for executing commands from JSON data provided via stdin.
"""

from typing import Dict, List

import click

from src.cql import CQLProcessor
from src.interactive import InteractiveHandler
from src.models.search_results import SearchResultItem
from src.stdin_processor import is_stdin_available, read_json_from_stdin
from src.tags import TagManager
from src.utils import sanitize_text


@click.command()
@click.option(
    "--abort-key", default="q", help="Key to abort all operations in interactive mode"
)
@click.pass_context
def from_stdin_json(ctx, abort_key):
    """
    Execute commands from JSON data provided via stdin.

    This command reads JSON data from stdin and executes the commands.
    The JSON data should have the same structure as the JSON file used with the from_json command.

    Example:
        cat examples/commands.json | ctag from_stdin_json
        echo '{"commands":[{"action":"add","cql_expression":"space = DOCS","tags":["tag1"]}]}' | ctag from_stdin_json
    """
    if not is_stdin_available():
        click.echo(
            "Error: No data provided via stdin. Use a pipe or redirect to provide JSON data."
        )
        return

    confluence = ctx.obj["CONFLUENCE"]
    dry_run = ctx.obj["DRY_RUN"]

    # Read commands from stdin
    try:
        commands = read_json_from_stdin()
    except Exception as e:
        click.echo(f"Error reading JSON from stdin: {str(e)}")
        return

    if not commands:
        click.echo("No valid commands found in the JSON data.")
        return

    click.echo(f"Found {len(commands)} commands in the JSON data.")

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
                        f"Would add tags {command.tags} to '{title}' (Space: {space})"
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
            interactive_handler = InteractiveHandler(
                default_response=True, abort_value=abort_key
            )

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
    excluded_ids = [
        page.content.id for page in excluded_pages if page.content and page.content.id
    ]
    return [
        page
        for page in pages
        if not (page.content and page.content.id and page.content.id in excluded_ids)
    ]
