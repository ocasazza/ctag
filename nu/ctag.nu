# Nushell wrapper for ctag
# Usage:
#   use nu/ctag.nu
#   ctag get "space = DOCS"

# Helper to run ctag and parse JSON
def run-ctag [cmd: string, args: list<string>, flags: record] {
    let json_flag = ["--format", "json"]
    let dry_run_flag = if ($flags.dry_run? | default false) { ["--dry-run"] } else { [] }

    # We use ^ctag to ensure we call the external binary
    ^ctag $cmd ...$args ...$json_flag ...$dry_run_flag | from json
}

export def get [
    query: string
    --tags-only (-t)
    --no-pages
] {
    let show_pages_arg = if $no_pages { "false" } else { "true" }
    let tags_only_flag = if $tags_only { ["--tags-only"] } else { [] }

    # Ensure show-pages can be set.
    # Note: Requires ctag to accept --show-pages=false or separate flag
    # process show-pages as string to ensure it's passed if logic allows

    ^ctag get $query --format json --show-pages $show_pages_arg ...$tags_only_flag | from json
}

export def add [
    query: string
    ...tags: string
    --dry-run (-d)
] {
    let dry_run_flag = if $dry_run { ["--dry-run"] } else { [] }
    ^ctag add $query ...$tags --format json ...$dry_run_flag | from json
}

export def remove [
    query: string
    ...tags: string
    --dry-run (-d)
] {
    let dry_run_flag = if $dry_run { ["--dry-run"] } else { [] }
    ^ctag remove $query ...$tags --format json ...$dry_run_flag | from json
}

export def replace [
    query: string
    old_tag: string
    new_tag: string
    --dry-run (-d)
] {
    let dry_run_flag = if $dry_run { ["--dry-run"] } else { [] }
    ^ctag replace $query $old_tag $new_tag --format json ...$dry_run_flag | from json
}

export def from-json [
    file: path
    --dry-run (-d)
] {
    let dry_run_flag = if $dry_run { ["--dry-run"] } else { [] }
    ^ctag from-json $file --format json ...$dry_run_flag | from json
}

export def from-stdin-json [
    --dry-run (-d)
] {
    let dry_run_flag = if $dry_run { ["--dry-run"] } else { [] }
    $in | to json | ^ctag from-stdin-json --format json ...$dry_run_flag | from json
}
