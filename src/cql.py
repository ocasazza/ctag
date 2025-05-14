#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
CQL (Confluence Query Language) processor module.

This module provides functionality for executing CQL queries and retrieving
matching Confluence pages as part of the ctag CLI tool.
"""

import logging
from typing import List, Dict, Optional, Any

logger = logging.getLogger(__name__)


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
                      expand: Optional[str] = None) -> List[Dict[str, Any]]:
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
            
            pages = results.get('results', [])
            logger.info(f"CQL query returned {len(pages)} results")
            
            return pages
        except Exception as e:
            logger.error(f"Error executing CQL query '{cql_expression}': {str(e)}")
            return []

    def get_all_results(self, cql_expression: str, 
                        expand: Optional[str] = None,
                        batch_size: int = 100) -> List[Dict[str, Any]]:
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
