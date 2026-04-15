#!/bin/bash
# example.sh
# Demonstrates best practices for skill scripts
# Usage: example.sh --name <name> [--greeting <greeting>]

set -euo pipefail

# Default values
NAME=""
GREETING="Hello"

# Help message
show_help() {
    cat << EOF
Usage: $(basename "$0") --name <name> [--greeting <greeting>]

Demonstrates script best practices with structured JSON output.

Options:
    --name       Name to greet (required)
    --greeting   Greeting message (default: Hello)
    --help       Show this help message

Examples:
    $(basename "$0") --name World
    $(basename "$0") --name World --greeting "Hi"
EOF
    exit 0
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --name)
            NAME="$2"
            shift 2
            ;;
        --greeting)
            GREETING="$2"
            shift 2
            ;;
        --help)
            show_help
            ;;
        *)
            echo "Error: Unknown option $1" >&2
            echo "Use --help for usage information" >&2
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$NAME" ]]; then
    echo "Error: --name is required" >&2
    exit 1
fi

# Main logic
main() {
    local message="${GREETING}, ${NAME}!"

    # Output structured JSON
    cat << EOF
{
  "status": "success",
  "message": "$message",
  "metadata": {
    "greeting": "$GREETING",
    "name": "$NAME",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  }
}
EOF
}

# Run main function
main
