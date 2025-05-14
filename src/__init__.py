# -*- coding: utf-8 -*-

"""
atool - A command line program for Confluence page management.

This package provides tools for synchronizing Confluence pages with a local
file system.
"""

__version__ = "0.1.0"

# Import main components for easier access
from src.engine import SyncEngine
from src.fs import LocalStorage


# Define public API
__all__ = [
    "SyncEngine",
    "LocalStorage",
]
