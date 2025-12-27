Output demo.gif

Set FontSize 20
Set Width 1500
Set Height 600
Set Padding 40
Set Theme "Catppuccin Macchiato"

# Clean up the prompt and set terminal width for consistent table formatting
Hide
Type "export PROMPT='> '"
Enter
Type "export PS1='> '"
Enter
# Set a width that fits comfortably in the pixels provided
Type "export COLUMNS=110"
Enter
Type "clear"
Enter
Show

Type "# Setup Environment"
Enter
Type "nix develop .#nu"
Enter
Sleep 3s

Type "# View current tags"
Enter
Type "ctag get 'id = 193593345'"
Enter
Sleep 5s

Type "# Add a few tags to a page"
Enter
Type "ctag add 'id = 193593345' 'test-tag-1' 'test-tag-2' 'test-tag-3' 'test-tag-foo'"
Sleep 1s
Enter
Sleep 1s

Type "# Show the new tags"
Enter
Type "ctag get 'id = 193593345'"
Enter
Sleep 3s

Type "# Replace tags using the regex pattern 'test-tag-\d*'"
Enter
Type "ctag replace --regex 'id = 193593345' 'test-tag-\d*'" 'test-tag-bar'
Sleep 1s
Enter
Sleep 1s

Type "# Show the tags we replaced"
Enter
Type "ctag get 'id = 193593345'"
Enter
Sleep 3s

Type "# Remove all the tags we added using the regex pattern 'test-.*'"
Enter
Type "ctag remove --regex 'id = 193593345' 'test-.*'
Sleep 1s
Enter
Sleep 1s

Type "# Show the tags we removed"
Enter
Type "ctag get 'id = 193593345'"
Enter
Sleep 5s
