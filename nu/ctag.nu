# Nushell wrapper for ctag
# Usage:
#   use nu/ctag.nu
#   ctag get "space = DOCS"

# Helper to format any ctag JSON output
def format-output [] {
    let results = $in
    # Check if 'details' column exists and is not empty
    if ('details' in ($results | columns)) and ($results.details | is-not-empty) {
        $results.details
        | update tags_added {|row| if ($row.tags_added | is-empty) { '[]' } else { $row.tags_added } }
        | update tags_removed {|row| if ($row.tags_removed | is-empty) { '[]' } else { $row.tags_removed } }
        | reject -o url page_id
    } else if ('total' in ($results | columns)) and ($results.total | is-not-empty) {
        # It's a summary result
         $results | reject -o details
    } else {
        # It's likely a list of pages (get command)
        $results | update title {|row|
            let url = ($row.url?)
            if ($url | is-empty) {
                $row.title
            } else {
                 # OSC 8 Hyperlink: <OSC>8;;<URL><ST><TEXT><OSC>8;;<ST>
                 # We use raw escape sequences because 'ansi link' isn't standard in all nu versions yet or simply constructing the string is robust.
                 # \e is \u{1b}
                 $"\u{1b}]8;;($url)\u{1b}\\($row.title)\u{1b}]8;;\u{1b}\\"
            }
        } | update tags {|row|
            # Return null for empty lists to show a blank cell instead of [list 0 items]
            if ($row.tags | is-empty) { '[]' } else { $row.tags }
        } | reject -o url id
    }
}

# Main generic wrapper
# This forwards all arguments to ctag-cli, ensuring we never have to manually update
# the wrapper when adding new flags or commands to the Rust binary.
export def --wrapped main [...args] {
    let input = $in
    let subcmd = ($args | get 0? | default "")

    let raw_output = if $subcmd == "from-stdin-json" {
        $input | to json | ^ctag-cli ...$args --format json
    } else {
        ^ctag-cli ...$args --format json
    }

    $raw_output | from json | format-output
}
