# ctag Usage


## Usage

### Basic Commands

#### Add tags to pages

```bash
ctag add "space = DOCS" tag1 tag2 tag3
```

#### Remove tags from pages

```bash
ctag remove "space = DOCS" old-tag
```

#### Replace tags

```bash
ctag replace "space = DOCS" old-tag=new-tag another-old=another-new
```

#### Get tags from pages

```bash
# Show all pages with their tags
ctag get "space = DOCS"

# Show only unique tags
ctag get "space = DOCS" --tags-only

# Output as JSON
ctag get "space = DOCS" --format json

# Save to file
ctag get "space = DOCS" --output-file results.json
```

### Regular Expression Support

#### Remove tags by pattern

```bash
# Remove all tags starting with "test-tag-"
ctag remove "space = DOCS" "test-tag-.*" --regex
```

#### Replace tags by pattern

```bash
# Replace any tag matching "id-[0-9]+" with "matched-id"
# Note: Use positional pairs (pattern replacement pattern replacement ...)
ctag replace --regex "space = DOCS" "id-[0-9]+" "matched-id"

# Multiple replacements
ctag replace --regex "space = DOCS" \
  "test-.*" "new-test" \
  "id-[0-9]+" "matched-id"
```

### Advanced Options

#### Interactive mode

Confirm each action before execution:

```bash
ctag add "space = DOCS" new-tag --interactive
```

#### Dry run

Preview changes without making modifications:

```bash
ctag --dry-run add "space = DOCS" new-tag
```

### Batch Operations

#### From JSON file

Create a JSON file with multiple commands:

```json
{
  "description": "Quarterly tag updates",
  "commands": [
    {
      "action": "add",
      "cql_expression": "space = DOCS AND lastmodified > -30d",
      "tags": ["recent", "q4-2024"],
      "interactive": false
    },
    {
      "action": "replace",
      "cql_expression": "space = DOCS",
      "tags": {
        "old-tag": "new-tag",
        "deprecated": "archived"
      },
      "interactive": true
    },
    {
      "action": "replace",
      "cql_expression": "space = DOCS",
      "tags": {
        "test-.*": "new-test",
        "id-[0-9]+": "matched-id"
      },
      "regex": true
    }
  ]
}
```

Execute the commands:

```bash
ctag from-json commands.json
```

#### From stdin

```bash
cat commands.json | ctag from-stdin-json
```

Or:

```bash
echo '{"commands":[{"action":"add","cql_expression":"space = DOCS","tags":["test"]}]}' | ctag from-stdin-json
```

## CQL Query

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
