#!/bin/bash

# Script to check modified files against main and fix copyright ordering
# Ensures that if "Stoolap Contributors" copyright exists, it's followed by "Oxibase Contributors"

set -e

echo "Checking modified files for copyright fixes..."

# Get list of modified .rs files compared to origin/main
modified_files=$(git diff --name-only origin/main | grep '\.rs$' || true)

if [ -z "$modified_files" ]; then
    echo "No modified .rs files found."
    exit 0
fi

echo "Found modified .rs files:"
echo "$modified_files"
echo ""

modified_count=0

for file in $modified_files; do
    if [ ! -f "$file" ]; then
        echo "Warning: $file not found, skipping."
        continue
    fi

    # Check if file contains Stoolap copyright
    if grep -q "// Copyright 2025 Stoolap Contributors" "$file"; then
        # Get line number of Stoolap copyright
        lineno=$(grep -n "// Copyright 2025 Stoolap Contributors" "$file" | head -1 | cut -d: -f1)

        # Get the next line
        next_lineno=$((lineno + 1))
        nextline=$(sed -n "${next_lineno}p" "$file" || true)

        # Check if next line is Oxibase copyright
        if [[ "$nextline" != "// Copyright 2025 Oxibase Contributors" ]]; then
            echo "Fixing copyright in $file (adding Oxibase after Stoolap on line $lineno)"

            # Insert Oxibase copyright after Stoolap line
            sed -i.bak "${lineno}a\\
// Copyright 2025 Oxibase Contributors" "$file"

            modified_count=$((modified_count + 1))
        else
            echo "$file already has correct copyright ordering"
        fi
    else
        echo "$file does not contain Stoolap copyright, skipping"
    fi
done

echo ""
if [ $modified_count -gt 0 ]; then
    echo "Modified $modified_count file(s). Backups created with .bak extension."
else
    echo "No files needed copyright fixes."
fi