#!/usr/bin/env bash
set -euo pipefail

# Ralph Loop - Vibe coding with Claude Code
# Usage: ./ralph/ralph.sh [max_iterations]
#
# No argument: infinite loop
# With argument: stops after N iterations

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RALPH_DIR="$SCRIPT_DIR"

cd "$PROJECT_DIR"

MAX_ITERATIONS="${1:-0}"
COUNT=0
LOGFILE="$RALPH_DIR/ralph.log"

echo "=== Ralph Loop started ==="
echo "Project: $PROJECT_DIR"
echo "Max iterations: ${MAX_ITERATIONS:-infinite}"
echo "Logs: tail -f $LOGFILE"
echo "Ctrl+C to stop"
echo ""

while true; do
    # Check if agent signaled completion
    if [[ -f "$RALPH_DIR/done.md" ]]; then
        echo "=== done.md detected — all tasks completed. Stopping. ==="
        break
    fi

    COUNT=$((COUNT + 1))
    echo "--- Iteration #${COUNT} ---"

    # Feed prompt to Claude Code (headless mode with visible output)
    claude --dangerously-skip-permissions --verbose -p "$(cat "$RALPH_DIR/PROMPT.md")" --output-format stream-json 2>&1 | tee -a "$LOGFILE"

    echo ""
    echo "--- Iteration #${COUNT} done ---"
    echo ""

    # Check if agent created done.md during this iteration
    if [[ -f "$RALPH_DIR/done.md" ]]; then
        echo "=== done.md detected — all tasks completed. Stopping. ==="
        break
    fi

    # Check max iterations
    if [[ "$MAX_ITERATIONS" -gt 0 && "$COUNT" -ge "$MAX_ITERATIONS" ]]; then
        echo "=== ${MAX_ITERATIONS} iterations completed. Stopping. ==="
        break
    fi

    # Brief pause between iterations
    sleep 2
done
