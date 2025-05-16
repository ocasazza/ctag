#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
CQL (Confluence Query Language) processor module.

This module provides functionality for executing CQL queries and retrieving
matching Confluence pages as part of the ctag CLI tool.
"""

import logging
import os
from typing import List, Dict, Optional, Any, Union
import json
from pydantic import BaseModel, create_model_from_schema

logger = logging.getLogger(__name__)

# Load JSON schema for SearchResultItem
SCHEMA_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), "examples")
SEARCH_RESULT_SCHEMA_PATH = os.path.join(SCHEMA_DIR, "search_result_schema.json")

# Check if schema file exists, if not use a default schema
if os.path.exists(SEARCH_RESULT_SCHEMA_PATH):
    with open(SEARCH_RESULT_SCHEMA_PATH, 'r') as f:
        search_result_schema = json.load(f)
else:
    raise Exception("search_result_schema.json file not found")

# Create Pydantic models from JSON schema
SearchResultItem = create_model_from_schema(
    schema=search_result_schema,
    model_name="SearchResultItem"
)

class CQLProcessor:
    """Processes CQL queries and retrieves matching Confluence pages."""

    def __init__(self, confluence):
        """Initialize the CQLProcessor with a Confluence client.

        Args:
            confluence: An authenticated Confluence client instance
        """
        self.confluence = confluence

    def execute_query(self, cql_expression: str, 
                      start: int = 0, 
                      limit: Optional[int] = None,
                      expand: Optional[str] = None) -> List[SearchResultItem]:
        """Execute a CQL query and return matching pages.

        Args:
            cql_expression: The CQL query expression
            start: Starting index for pagination
            limit: Maximum number of results to return
            expand: Comma-separated list of properties to expand

        Returns:
            List of page dictionaries matching the query
        """
        if not expand:
            # Default expansions to get useful page information
            expand = "space,metadata.labels,version"

        try:
            logger.info(f"Executing CQL query: {cql_expression}")
            results = self.confluence.cql(
                cql=cql_expression,
                start=start,
                limit=limit,
                expand=expand
            )
            
            # Parse the JSON array string into a Python list of dictionaries
            # Convert each result item to a SearchResultItem object
            pages: List[SearchResultItem] = [
                SearchResultItem(item) for item in json.loads(results.get('results', []))
            ]

            logger.info(f"CQL query returned {len(pages)} results")
            
            return pages

        except Exception as e:
            logger.error(f"Error executing CQL query '{cql_expression}': {str(e)}")
            return []

    def get_all_results(self, cql_expression: str, 
                        expand: Optional[str] = None,
                        batch_size: int = 100) -> List[SearchResultItem]:
        """Get all results for a CQL query, handling pagination.

        Args:
            cql_expression: The CQL query expression
            expand: Comma-separated list of properties to expand
            batch_size: Number of results to fetch per request

        Returns:
            List of all page dictionaries matching the query
        """
        all_pages = []
        start = 0
        
        while True:
            batch = self.execute_query(
                cql_expression=cql_expression,
                start=start,
                limit=batch_size,
                expand=expand
            )
            
            if not batch:
                break
                
            all_pages.extend(batch)
            
            if len(batch) < batch_size:
                break
                
            start += batch_size
            
        return all_pages
