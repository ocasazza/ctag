#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Remove command module for the ctag CLI tool.

This module defines the 'remove' command for removing tags from Confluence pages.
"""

import click
from src.cql import CQLProcessor
from src.models.search_results import SearchResultItem
from src.tags import TagManager
from src.interactive import InteractiveHandler
from src.utils import sanitize_text
from typing import List


@click.command()
@click.argument('cql_expression')
@click.argument('tags', nargs=-1, required=True)
@click.option('--interactive', is_flag=True, help="Confirm each action interactively")
@click.option('--abort-key', default='q', help="Key to abort all operations in interactive mode")
@click.option('--cql-exclude', required=False, help="CQL expression to match pages that should be excluded from this operation")
@click.pass_context
def remove(ctx, cql_expression, tags, interactive, abort_key, cql_exclude):
    """
    Remove tags from pages matching CQL expression.
    
    CQL_EXPRESSION is a Confluence Query Language expression.
    TAGS are one or more tags to remove from matching pages.
    
    Example:
        ctag remove "space = DOCS" tag1 tag2
    """
    confluence = ctx.obj['CONFLUENCE']
    dry_run = ctx.obj['DRY_RUN']
    
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
            click.echo(f"Excluded {original_count - len(pages)} pages. {len(pages)} pages remaining.")
    
    if dry_run:
        click.echo("DRY RUN: No changes will be made.")
        for page in pages:
            title = sanitize_text(page.title if page.title else 'Unknown')
            # Handle both object and dictionary access for resultGlobalContainer
            if hasattr(page, 'resultGlobalContainer'):
                if isinstance(page.resultGlobalContainer, dict):
                    space = page.resultGlobalContainer.get('title', 'Unknown')
                else:
                    space = page.resultGlobalContainer.title if hasattr(page.resultGlobalContainer, 'title') else 'Unknown'
            else:
                space = 'Unknown'
            click.echo(f"Would remove tags {list(tags)} from '{title}' (Space: {space})")
        return
    
    # Set up interactive handler if needed
    interactive_handler = None
    if interactive:
        interactive_handler = InteractiveHandler(default_response=True, abort_value=abort_key)
    
    # Process the pages
    results = tag_manager.process_pages(
        pages=pages,
        action='remove',
        tags=list(tags),
        interactive=interactive,
        interactive_handler=interactive_handler
    )
    
    # Display results
    click.echo(f"\nResults:")
    click.echo(f"  Total pages: {results['total']}")
    click.echo(f"  Processed: {results['processed']}")
    click.echo(f"  Skipped: {results['skipped']}")
    click.echo(f"  Successful: {results['success']}")
    click.echo(f"  Failed: {results['failed']}")


def filter_excluded_pages(pages: List[SearchResultItem], excluded_pages: List[SearchResultItem]) -> List[SearchResultItem]:
    """Filter out pages that are in the excluded_pages list based on page ID.
    
    Args:
        pages: List of pages to filter
        excluded_pages: List of pages to exclude
        
    Returns:
        Filtered list of pages
    """
    excluded_ids = [page.content.id for page in excluded_pages if page.content and page.content.id]
    return [page for page in pages if not (page.content and page.content.id and page.content.id in excluded_ids)]
