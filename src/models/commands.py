#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Command models module.

This module provides Pydantic models for command validation and type checking,
generated from the commands JSON schema.
"""

import json
import os
from typing import Any, Dict, List, Optional, Union

from pydantic import BaseModel

# Load the commands schema directly
schema_path = os.path.join(os.path.dirname(__file__), "./commands.json")
with open(schema_path, "r") as f:
    commands_schema = json.load(f)


class CommandModel(BaseModel):
    """Model for individual commands."""

    action: str
    cql_expression: str
    tags: Union[List[str], Dict[str, str]]  # List for add/remove, Dict for replace
    interactive: Optional[bool] = False
    cql_exclude: Optional[str] = None


class CommandsFileModel(BaseModel):
    """Model for the entire commands file."""

    description: Optional[str] = None
    commands: List[CommandModel]


# Export the models for use in other modules
__all__ = ["CommandModel", "CommandsFileModel"]
