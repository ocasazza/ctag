#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Interactive handler module for CLI confirmations.

This module provides functionality for interactive confirmations
during bulk tag operations as part of the ctag CLI tool.
"""

import logging
from typing import Any, Optional

import click

from src.utils import sanitize_text

logger = logging.getLogger(__name__)


class InteractiveHandler:
    """Handles interactive confirmations for CLI operations."""

    def __init__(
        self, default_response: bool = False, abort_value: Optional[str] = None
    ):
        """Initialize the InteractiveHandler.

        Args:
            default_response: Default response if user just presses Enter
            abort_value: Special value that will abort the entire operation if entered
        """
        self.default_response = default_response
        self.abort_value = abort_value
        self.aborted = False

    def confirm_action(self, item: Any, action_description: str) -> bool:
        """Ask user to confirm an action on an item.

        Args:
            item: The item to perform the action on (will be displayed to user)
            action_description: Description of the action to perform

        Returns:
            True if user confirmed, False otherwise
        """
        if self.aborted:
            return False

        # Sanitize the item if it's a string
        if isinstance(item, str):
            item = sanitize_text(item)

        prompt = f"{action_description} {item}"

        if self.abort_value:
            prompt += f" (Enter '{self.abort_value}' to abort all remaining operations)"

        # Use click's confirmation prompt
        response = click.prompt(
            prompt, type=str, default="y" if self.default_response else "n"
        ).lower()

        # Check for abort value
        if self.abort_value and response.lower() == self.abort_value.lower():
            click.echo("Aborting all remaining operations.")
            self.aborted = True
            return False

        # Check for yes/no responses
        if response in ("y", "yes"):
            return True
        elif response in ("n", "no"):
            return False
        else:
            # Default to the default response for any other input
            return self.default_response

    def reset(self):
        """Reset the aborted state."""
        self.aborted = False
