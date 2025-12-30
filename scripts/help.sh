#!/bin/bash

# This script displays help information for the Makefile.
# Usage: ./help.sh Makefile

# Set colors for output
col_off='\033[0m'
target_col='\033[36m'
variable_col='\033[93m'
grey='\033[90m'

# Main function to display help information
help() {
    # Get version from Cargo.toml
    version=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

    # Display logo and version
    cat <<EOF

          ██████
       ███████████             ███████                 ███  █████
     ███████      ██         ███░░░░░███              ░░░  ░░███
    ██████         ██       ███     ░░███ █████ █████ ████  ░███████   ██████    █████   ██████
   █████           ███     ░███      ░███░░███ ░░███ ░░███  ░███░░███ ░░░░░███  ███░░   ███░░███
   ████            ███     ░███      ░███ ░░░█████░   ░███  ░███ ░███  ███████ ░░█████ ░███████
   ███           █████     ░░███     ███   ███░░░███  ░███  ░███ ░███ ███░░███  ░░░░███░███░░░
    ██         ██████       ░░░███████░   █████ █████ █████ ████████ ░░████████ ██████ ░░██████
     ██      ███████          ░░░░░░░    ░░░░░ ░░░░░ ░░░░░ ░░░░░░░░   ░░░░░░░░ ░░░░░░   ░░░░░░
       ███████████
          █████

Oxibase v$version

EOF

    # Display usage information
    echo "Usage:"
    printf "  make %b[target]%b %b[variables]%b\n\n" "$target_col" "$col_off" "$variable_col" "$col_off"

    # Display targets information
    _help_targets "$1"

    # Display variables information
    _help_variables "$1"

}

# Function to display targets information
_help_targets() {
    dev_targets=""
    docs_targets=""
    release_targets=""
    run_targets=""
    other_targets=""
    dev_col='\033[32m'
    docs_col='\033[34m'
    release_col='\033[31m'
    run_col='\033[35m'
    other_col='\033[36m'

    while IFS='|' read -r target group desc; do
        case $group in
            dev) dev_targets+="$target|$desc\n" ;;
            docs) docs_targets+="$target|$desc\n" ;;
            release) release_targets+="$target|$desc\n" ;;
            run) run_targets+="$target|$desc\n" ;;
            other) other_targets+="$target|$desc\n" ;;
        esac
    done < <(awk '
/^#/ {
    if ($0 ~ /^# \[[^\]]+\] .+/) {
        start = index($0, "[")
        end = index($0, "]")
        group = substr($0, start+1, end-start-1)
        desc_start = index($0, "] ") + 2
        desc = substr($0, desc_start)
        getline
        if ($0 ~ /^[a-zA-Z0-9._-]+:/) {
            target = $1
            sub(/:$/, "", target)
            print target "|" group "|" desc
        }
    }
}' "$1" | sort)

    groups=("dev" "docs" "release" "run" "other")
    bold='\033[1m'
    for group in "${groups[@]}"; do
        targets_var="${group}_targets"
        color_var="${group}_col"
        if [[ -n ${!targets_var} ]]; then
            case $group in
                dev) group_name="Development targets" ;;
                docs) group_name="Documentation targets" ;;
                release) group_name="Release targets" ;;
                run) group_name="Runtime targets" ;;
                other) group_name="Other targets" ;;
            esac
            echo -e "${bold}${group_name}:${col_off}"
            echo -e "${!targets_var}" | sed '/^$/d' | while IFS='|' read -r tgt dsc; do
                printf "  %b%-30s%b%s\n" "${!color_var}" "$tgt" "$col_off" "$dsc"
            done
            echo ""
        fi
    done
}

# Function to display variables information
_help_variables() {
    echo "Variable(s):"
    grep -E '^[a-zA-Z0-9_-]+ [:=!?+]?=.*?##.*$' "$1" | while read -r line; do
        variable=${line%% *}
        default=${line#*= }
        default=${default%%##*}
        description=${line##*## }
        printf "  %b%-30s%b%s %b(default: %s)%b\n" "$variable_col" "$variable" "$col_off" "$description" "$grey" "$default" "$col_off"
    done
    echo ""
}

# Call main function
help "$1"
