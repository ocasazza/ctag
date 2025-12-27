# Disable banner first
$env.config = ( $env.config? | default {} | upsert show_banner false )

print "Loading ctag environment..."

# Load user config if it was found


# Load the ctag module
# Crucial: Variable expansion happens here in Bash to put the literal path in the file
use "/Users/casazza/Repositories/ctag/nu/ctag.nu"

print "ctag module loaded."
