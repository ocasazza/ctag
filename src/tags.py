#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Tag management module for Confluence pages.

This module provides functionality for adding, removing, and replacing tags
on Confluence pages in bulk as part of the ctag CLI tool.
"""

import logging
from typing import List, Dict, Set, Optional
from src.utils import sanitize_text

logger = logging.getLogger(__name__)


class TagManager:
    """Manages tags on Confluence pages."""

    def __init__(self, confluence):
        """Initialize the TagManager with a Confluence client.

        Args:
            confluence: An authenticated Confluence client instance
        """
        self.confluence = confluence

    def get_page_tags(self, page_id: str) -> List[str]:
        """Get all tags for a specific page.

        Args:
            page_id: The ID of the Confluence page

        Returns:
            A list of tag names
        """
        try:
            labels = self.confluence.get_page_labels(page_id)
            return [label['name'] for label in labels.get('results', [])]
        except Exception as e:
            logger.error(f"Error getting tags for page {page_id}: {str(e)}")
            return []

    def add_tags(self, page_id: str, tags: List[str]) -> bool:
        """Add tags to a Confluence page.

        Args:
            page_id: The ID of the Confluence page
            tags: List of tags to add

        Returns:
            True if successful, False otherwise
        """
        success = True
        for tag in tags:
            try:
                self.confluence.set_page_label(page_id, tag)
                logger.info(f"Added tag '{tag}' to page {page_id}")
            except Exception as e:
                logger.error(f"Error adding tag '{tag}' to page {page_id}: {str(e)}")
                success = False
        return success

    def remove_tags(self, page_id: str, tags: List[str]) -> bool:
        """Remove tags from a Confluence page.

        Args:
            page_id: The ID of the Confluence page
            tags: List of tags to remove

        Returns:
            True if successful, False otherwise
        """
        success = True
        for tag in tags:
            try:
                self.confluence.remove_page_label(page_id, tag)
                logger.info(f"Removed tag '{tag}' from page {page_id}")
            except Exception as e:
                logger.error(f"Error removing tag '{tag}' from page {page_id}: {str(e)}")
                success = False
        return success

    def replace_tags(self, page_id: str, tag_mapping: Dict[str, str]) -> bool:
        """Replace tags on a Confluence page.

        Args:
            page_id: The ID of the Confluence page
            tag_mapping: Dictionary mapping old tags to new tags

        Returns:
            True if successful, False otherwise
        """
        current_tags = self.get_page_tags(page_id)
        success = True

        for old_tag, new_tag in tag_mapping.items():
            if old_tag in current_tags:
                try:
                    # Remove the old tag
                    self.confluence.remove_page_label(page_id, old_tag)
                    # Add the new tag
                    self.confluence.set_page_label(page_id, new_tag)
                    logger.info(f"Replaced tag '{old_tag}' with '{new_tag}' on page {page_id}")
                except Exception as e:
                    logger.error(f"Error replacing tag '{old_tag}' with '{new_tag}' on page {page_id}: {str(e)}")
                    success = False

        return success

    def process_pages(self, pages: List[dict], action: str, 
                      tags: Optional[List[str]] = None, 
                      tag_mapping: Optional[Dict[str, str]] = None,
                      interactive: bool = False,
                      interactive_handler=None) -> Dict[str, int]:
        """Process tags on multiple pages.

        Args:
            pages: List of page dictionaries from CQL query
            action: Action to perform ('add', 'remove', or 'replace')
            tags: List of tags for add/remove actions
            tag_mapping: Dictionary mapping old tags to new tags for replace action
            interactive: Whether to confirm each action interactively
            interactive_handler: Handler for interactive confirmations

        Returns:
            Dictionary with counts of successful and failed operations
        """
        results = {
            'total': len(pages),
            'processed': 0,
            'skipped': 0,
            'success': 0,
            'failed': 0
        }

        for page in pages:
            # Try to get the page ID directly or from the content object
            page_id = page.get('id')
            if not page_id and 'content' in page:
                page_id = page.get('content', {}).get('id')
                
            page_title = sanitize_text(page.get('title', 'Unknown'))
            page_space = page.get('space', {}).get('key', 'Unknown')
            
            if not page_id:
                logger.warning(f"Skipping page with no ID: {page_title}")
                results['skipped'] += 1
                continue

            # Format the action description for confirmation
            if action == 'add':
                action_desc = f"Add tags {tags} to"
            elif action == 'remove':
                action_desc = f"Remove tags {tags} from"
            elif action == 'replace':
                action_desc = f"Replace tags {list(tag_mapping.keys())} with {list(tag_mapping.values())} on"
            else:
                action_desc = "Process"

            # Confirm action if in interactive mode
            if interactive and interactive_handler:
                page_info = f"'{page_title}' (Space: {page_space}, ID: {page_id})"
                if not interactive_handler.confirm_action(page_info, action_desc):
                    logger.info(f"Skipped {action} tags on page {page_id} ({page_title})")
                    results['skipped'] += 1
                    continue

            # Perform the requested action
            success = False
            try:
                if action == 'add':
                    success = self.add_tags(page_id, tags)
                elif action == 'remove':
                    success = self.remove_tags(page_id, tags)
                elif action == 'replace':
                    success = self.replace_tags(page_id, tag_mapping)
            except Exception as e:
                logger.error(f"Error processing tags on page {page_id} ({page_title}): {str(e)}")
                success = False

            results['processed'] += 1
            if success:
                results['success'] += 1
            else:
                results['failed'] += 1

        return results
