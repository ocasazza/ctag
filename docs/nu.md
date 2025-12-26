# Nushell Integration

`ctag` provides a module to integrate with Nushell, allowing you to manipulate Confluence tags using Nushell's powerful structured data pipelines.

## Setup

Import the module from the repository root:

```nu
use nu/ctag.nu
```

## Basic Examples

### Querying Tags

Get all pages matching a specific CQL query. This returns a table with `id`, `title`, `space`, `tags`, and `ancestors`.

```nu
ctag get "space = DOCS"
```

**Filtering with Nushell:**

Filter for pages that have specific tags:

```nu
ctag get "space = DOCS" | where ($it.tags | any { |x| $x == "outdated" })
```

Filter for pages that have *no* tags:

```nu
ctag get "space = DOCS" | where ($it.tags | is-empty)
```

**Tags Only:**

Get a list of all unique tags used in a space:

```nu
ctag get "space = DOCS" --tags-only
```

### Modifying Tags

**Add Tags:**

Add a single tag to matching pages:

```nu
ctag add "space = DOCS" "reviewed"
```

**Replace Tags:**

Rename a tag across a space:

```nu
ctag replace "space = DOCS" "wip" "work-in-progress"
```

## Advanced Pipelines

These examples demonstrate the power of combining `ctag` with Nushell's data processing capabilities.

### 1. Tag Usage Statistics

Generate a frequency count of all tags in a space to see which are most common.

```nu
ctag get "space = DOCS"
    | get tags
    | flatten
    | uniq -c
    | sort-by count -r
```

### 2. Audit for Unauthorized Tags

Find pages that contain tags *not* present in an approved list.

```nu
let allowed_tags = ["official", "draft", "reviewed", "archive"]

ctag get "space = DOCS"
    | link {
        # Create a new column 'invalid_tags' containing any tag not in the allowed list
        invalid_tags: ($it.tags | where ($it not-in $allowed_tags))
    }
    | where ($it.invalid_tags | is-not-empty)
    | select title invalid_tags
```

### 3. Join with External Data (Page Owners)

Imagine you have a CSV file `owners.csv` mapping page titles to teams:

| title | team |
| --- | --- |
| Getting Started | Engineering |
| API Reference | Engineering |
| HR Policy | People |

You can join this data with `ctag` results to find, for example, which pages owned by "Engineering" are still marked as "draft".

```nu
# Load local CSV
let owners = open owners.csv

# Fetch tags and join
ctag get "space = DOCS"
    | join $owners title
    | where team == "Engineering"
    | where ("draft" in $it.tags)
    | select title tags
```
