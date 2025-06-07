#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Text utility functions for the ctag CLI tool.
"""

import html
import re


def clean_title(title: str) -> str:
    """Clean up a title by removing HTML highlighting and escape sequences.

    Args:
        title: The title to clean

    Returns:
        The cleaned title
    """
    # Remove highlighting tags
    title = re.sub(r"@@@hl@@@", "", title)
    title = re.sub(r"@@@endhl@@@", "", title)

    # Decode HTML entities using Python's html module
    title = html.unescape(title)

    return title


def sanitize_text(text: str) -> str:
    """Sanitize text by removing formatting tags and decoding HTML entities.

    This is a more generic function that can be used for any text, not just titles.

    Args:
        text: The text to sanitize

    Returns:
        The sanitized text
    """
    if not text:
        return ""

    # Remove common formatting tags
    text = re.sub(r"@@@\w+@@@", "", text)  # Remove any @@@tag@@@ style tags

    # Decode HTML entities
    text = html.unescape(text)

    # Remove any remaining HTML tags
    text = re.sub(r"<[^>]+>", "", text)

    return text
