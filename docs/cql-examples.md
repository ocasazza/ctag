# CQL (Confluence Query Language) Examples

This document provides examples of CQL expressions that can be used with the tag management CLI.

## Basic CQL Syntax

CQL (Confluence Query Language) is a query language used to search for content in Confluence. It's similar to SQL but designed specifically for Confluence content.

Basic syntax:
```
field operator value
```

Multiple conditions can be combined with `AND` and `OR`:
```
field1 operator value1 AND field2 operator value2
```

## Common Fields

- `space`: The space key
- `title`: The page title
- `text`: The page content
- `creator`: The username of the page creator
- `lastmodifier`: The username of the last person to modify the page
- `created`: The creation date
- `lastmodified`: The last modification date
- `label`: The page labels/tags

## Common Operators

- `=`: Equals
- `!=`: Not equals
- `~`: Contains
- `!~`: Does not contain
- `>`, `>=`, `<`, `<=`: Greater than, greater than or equal, less than, less than or equal (for dates)
- `IN`: Value is in a list
- `NOT IN`: Value is not in a list

## Date Formats

Dates can be specified in several ways:
- Absolute: `YYYY-MM-DD`
- Relative: `-Nd` (N days ago), `-Nw` (N weeks ago), `-Nm` (N months ago)

## Examples for Tag Management

### Finding Pages by Space

```
space = "DOCS"
```
Finds all pages in the DOCS space.

### Finding Pages by Title

```
title ~ "Project"
```
Finds all pages with "Project" in the title.

```
title = "Home Page"
```
Finds pages with the exact title "Home Page".

### Finding Pages by Content

```
text ~ "deprecated"
```
Finds pages containing the word "deprecated".

### Finding Pages by Creator or Modifier

```
creator = "jsmith"
```
Finds pages created by user "jsmith".

```
lastmodifier = "mjones"
```
Finds pages last modified by user "mjones".

### Finding Pages by Date

```
created > "2023-01-01"
```
Finds pages created after January 1, 2023.

```
lastmodified < "-30d"
```
Finds pages not modified in the last 30 days.

### Finding Pages by Labels/Tags

```
label = "documentation"
```
Finds pages with the "documentation" label.

```
label IN ("urgent", "important")
```
Finds pages with either the "urgent" or "important" label.

```
label != "draft"
```
Finds pages that don't have the "draft" label.

### Combining Conditions

```
space = "DOCS" AND lastmodified > "-7d"
```
Finds pages in the DOCS space modified in the last 7 days.

```
(label = "review" OR label = "feedback") AND creator = "jsmith"
```
Finds pages with either the "review" or "feedback" label, created by "jsmith".

## Using with the Tag Management CLI

### Add Tags Example

```bash
ctag tags add "space = DOCS AND lastmodified > -30d" important current
```
Adds the tags "important" and "current" to all pages in the DOCS space that were modified in the last 30 days.

### Remove Tags Example

```bash
ctag tags remove "label = outdated AND space = DOCS" outdated --interactive
```
Interactively removes the "outdated" tag from pages in the DOCS space that have the "outdated" tag.

### Replace Tags Example

```bash
ctag tags replace "space = DOCS AND label = oldtag" oldtag=newtag
```
Replaces the "oldtag" tag with "newtag" on all pages in the DOCS space that have the "oldtag" tag.

## Advanced Examples

```bash
ctag tags add "space = DOCS AND creator = currentUser()" owner
```
Adds the "owner" tag to all pages in the DOCS space created by the current user.

```bash
ctag tags remove "type = page AND lastmodified < -90d AND label = current" current
```
Removes the "current" tag from all pages (not blog posts) that haven't been modified in the last 90 days and have the "current" tag.

```bash
ctag tags replace "space IN (DOCS, PROJ, TEAM) AND text ~ 'deprecated API'" deprecated=legacy
```
Replaces the "deprecated" tag with "legacy" on all pages in the DOCS, PROJ, or TEAM spaces that mention "deprecated API" in their content.
