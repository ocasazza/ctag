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

Type "# View current tags"
Enter
Type "ctag get 'id = 193593345'"
Enter
Sleep 3s

Type "# Remove tags matching 'test-tag-.*'"
Enter
Type "ctag remove --regex 'id = 193593345' 'test-tag-.*' --dry-run"
Sleep 1s
Enter
Sleep 4s

Type "# Replace tags matching a numeric pattern"
Enter
Type "ctag replace --regex 'id = 193593345' 'id-[0-9]+=matched-id' --dry-run"
Sleep 1s
Enter
Sleep 5s
