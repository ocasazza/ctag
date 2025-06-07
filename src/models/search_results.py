#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Search results models module.

This module provides Pydantic models for Confluence search results,
generated from the search_results JSON schema.
"""

import json
import os
from typing import Any, Dict

from src.utils.pydantic_utils import create_model_from_schema

# Load the search results schema directly
schema_path = os.path.join(os.path.dirname(__file__), "./search_results.json")
with open(schema_path, "r") as f:
    search_results_schema = json.load(f)

# Create Pydantic model from JSON schema
# This automatically generates a model with all the fields defined in the
# schema
SearchResultItem = create_model_from_schema(schema=search_results_schema, model_name="SearchResultItem")

# Export the model for use in other modules
__all__ = ["SearchResultItem"]
