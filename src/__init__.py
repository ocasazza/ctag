# -*- coding: utf-8 -*-

"""
ctag - A command line tool for managing tags on Confluence pages in bulk.

This package provides tools for managing Confluence page tags, including
tag operations and CQL query processing.
"""

__version__ = "0.1.0"

# Import main components for easier access
from src.tags import TagManager
from src.cql import CQLProcessor
from src.interactive import InteractiveHandler

# Define public API
__all__ = [
    "TagManager",
    "CQLProcessor",
    "InteractiveHandler",
]
