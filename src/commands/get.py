#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Get command module for the ctag CLI tool.

This module defines the 'get' command for retrieving and displaying tags
from Confluence pages matching a CQL expression.
"""

import json
from typing import Dict, List, Optional, Set

import click

from src.cql import CQLProcessor
from src.interactive import InteractiveHandler
from src.models.search_results import SearchResultItem
from src.tags import TagManager
from src.utils import sanitize_text


@click.command()
@click.argument("cql_expression")
@click.option(
    "--format",
    "output_format",
    type=click.Choice(["table", "json"], case_sensitive=False),
    default="table",
    help="Output format",
)
@click.option(
    "--show-pages/--no-show-pages",
    default=True,
    help="Include page titles and spaces in output",
)
@click.option("--tags-only", is_flag=True, help="Show only unique tags across all pages")
@click.option("--interactive", is_flag=True, help="Browse results interactively")
@click.option(
    "--abort-key",
    default="q",
    help="Key to abort all operations in interactive mode",
)
@click.option(
    "--cql-exclude",
    required=False,
    help="CQL expression to match pages that should be excluded",
)
@click.option("--output-file", type=click.Path(), help="Save results to file")
@click.pass_context
def get(
    ctx,
    cql_expression,
    output_format,
    show_pages,
    tags_only,
    interactive,
    abort_key,
    cql_exclude,
    output_file,
):
    """
    Get tags from pages matching CQL expression.

    CQL_EXPRESSION is a Confluence Query Language expression.

    Examples:
        ctag get "space = DOCS"
        ctag get "space = DOCS" --tags-only
        ctag get "lastmodified > -7d" --format json
        ctag get "title ~ 'Project*'" --interactive
    """
    confluence = ctx.obj["CONFLUENCE"]

    # Initialize our processors
    cql_processor = CQLProcessor(confluence)
    tag_manager = TagManager(confluence)

    # Get matching pages
    click.echo(f"Finding pages matching: {cql_expression}")
    pages = cql_processor.get_all_results(cql_expression)

    if not pages:
        click.echo("No pages found matching the CQL expression.")
        return

    click.echo(f"Found {len(pages)} matching pages.")

    # Apply exclusion if specified
    if cql_exclude:
        click.echo(f"Finding pages to exclude: {cql_exclude}")
        excluded_pages = cql_processor.get_all_results(cql_exclude)
        if excluded_pages:
            original_count = len(pages)
            pages = filter_excluded_pages(pages, excluded_pages)
            click.echo(
                f"Excluded {
        original_count -
        len(pages)} pages. {
            len(pages)} pages remaining."
            )

    # Collect page data with tags
    page_data = []
    all_tags = set()

    click.echo("Retrieving tags for pages...")

    for page in pages:
        # Get the page ID from the content object or dictionary
        page_id = None
        if page.content:
            if isinstance(page.content, dict):
                page_id = page.content.get("id")
            elif hasattr(page.content, "id"):
                page_id = page.content.id

        page_title = sanitize_text(page.title if page.title else "Unknown")

        # Get space information if available
        page_space = "Unknown"
        if hasattr(page, "resultGlobalContainer"):
            if isinstance(page.resultGlobalContainer, dict):
                page_space = page.resultGlobalContainer.get("title", "Unknown")
            elif hasattr(page.resultGlobalContainer, "title"):
                page_space = page.resultGlobalContainer.title

        if not page_id:
            click.echo(f"Warning: Skipping page with no ID: {page_title}", err=True)
            continue

        # Get tags for this page
        page_tags = tag_manager.get_page_tags(page_id)
        all_tags.update(page_tags)

        page_info = {
            "id": page_id,
            "title": page_title,
            "space": page_space,
            "tags": page_tags,
        }
        page_data.append(page_info)

    # Handle interactive mode
    if interactive:
        interactive_handler = InteractiveHandler(default_response=True, abort_value=abort_key)
        page_data = handle_interactive_browsing(page_data, interactive_handler)

    # Generate output based on format and options
    if tags_only:
        output_content = format_tags_only_output(all_tags, output_format)
    else:
        output_content = format_page_data_output(page_data, output_format, show_pages)

    # Output results
    if output_file:
        with open(output_file, "w", encoding="utf-8") as f:
            f.write(output_content)
        click.echo(f"Results saved to {output_file}")
    else:
        click.echo(output_content)

    # Display summary
    click.echo(f"\nSummary:")
    click.echo(f"  Total pages processed: {len(page_data)}")
    click.echo(f"  Unique tags found: {len(all_tags)}")


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
    excluded_ids = []
    for page in excluded_pages:
        if page.content:
            if isinstance(page.content, dict):
                page_id = page.content.get("id")
                if page_id:
                    excluded_ids.append(page_id)
            elif hasattr(page.content, "id"):
                if page.content.id:
                    excluded_ids.append(page.content.id)

    filtered_pages = []
    for page in pages:
        include = True
        if page.content:
            if isinstance(page.content, dict):
                page_id = page.content.get("id")
                if page_id and page_id in excluded_ids:
                    include = False
            elif hasattr(page.content, "id"):
                if page.content.id and page.content.id in excluded_ids:
                    include = False

        if include:
            filtered_pages.append(page)

    return filtered_pages


def handle_interactive_browsing(page_data: List[Dict], interactive_handler) -> List[Dict]:
    """Handle interactive browsing of page results.

    Args:
        page_data: List of page dictionaries
        interactive_handler: Handler for interactive confirmations

    Returns:
        Filtered list of pages based on user selections
    """
    selected_pages = []

    for page in page_data:
        page_info = f"'{
    page['title']}' (Space: {
        page['space']}, Tags: {
            ', '.join(
                page['tags']) if page['tags'] else 'None'})"
        if interactive_handler.confirm_action(page_info, "Include in results"):
            selected_pages.append(page)

    return selected_pages


def format_tags_only_output(tags: Set[str], output_format: str) -> str:
    """Format output showing only unique tags.

    Args:
        tags: Set of unique tags
        output_format: Output format ('table', 'json')

    Returns:
        Formatted output string
    """
    sorted_tags = sorted(tags)

    if output_format == "json":
        return json.dumps(sorted_tags, indent=2, ensure_ascii=False)
    else:  # table format
        if not sorted_tags:
            return "No tags found."

        output_lines = ["Tags found:"]
        output_lines.append("=" * 50)
        for tag in sorted_tags:
            output_lines.append(f"  {tag}")
        return "\n".join(output_lines)


def format_page_data_output(page_data: List[Dict], output_format: str, show_pages: bool) -> str:
    """Format output showing page data with tags.

    Args:
        page_data: List of page dictionaries
        output_format: Output format ('table', 'json')
        show_pages: Whether to include page information

    Returns:
        Formatted output string
    """
    if output_format == "json":
        if show_pages:
            return json.dumps(page_data, indent=2, ensure_ascii=False)
        else:
            # Extract just tags
            all_tags = []
            for page in page_data:
                all_tags.extend(page["tags"])
            return json.dumps(sorted(set(all_tags)), indent=2, ensure_ascii=False)

    else:  # table format
        if not page_data:
            return "No pages found."

        output_lines = []

        if show_pages:
            # Calculate column widths
            max_title_len = max(len(page["title"]) for page in page_data)
            max_space_len = max(len(page["space"]) for page in page_data)
            title_width = min(max_title_len, 50)  # Cap at 50 chars
            space_width = min(max_space_len, 20)  # Cap at 20 chars

            # Header
            header = f"{'Title':<{title_width}} {'Space':<{space_width}} Tags"
            output_lines.append(header)
            output_lines.append("=" * len(header))

            # Data rows
            for page in page_data:
                title = page["title"][:title_width]
                space = page["space"][:space_width]
                tags = ", ".join(page["tags"]) if page["tags"] else "(no tags)"

                row = f"{title:<{title_width}} {space:<{space_width}} {tags}"
                output_lines.append(row)
        else:
            # Show only unique tags
            all_tags = set()
            for page in page_data:
                all_tags.update(page["tags"])

            if all_tags:
                output_lines.append("Tags found:")
                output_lines.append("=" * 50)
                for tag in sorted(all_tags):
                    output_lines.append(f"  {tag}")
            else:
                output_lines.append("No tags found.")

        return "\n".join(output_lines)
