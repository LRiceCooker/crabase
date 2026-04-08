#!/usr/bin/env bash
set -euo pipefail

# Ralph Loop - Boucle de vibe coding avec Claude Code
# Usage: ./ralph.sh [nombre_iterations]
#
# Sans argument : boucle infinie
# Avec argument  : s'arrête après N itérations

MAX_ITERATIONS="${1:-0}"
COUNT=0
LOGFILE="ralph.log"

echo "=== Ralph Loop démarré ==="
echo "Iterations max: ${MAX_ITERATIONS:-infini}"
echo "Logs: tail -f ${LOGFILE}"
echo "Ctrl+C pour arrêter"
echo ""

while true; do
    COUNT=$((COUNT + 1))
    echo "--- Iteration #${COUNT} ---"

    # Nourrir le prompt à Claude Code (mode headless avec output visible)
    # Seul le JSON de claude va dans le logfile
    claude --dangerously-skip-permissions --verbose -p "$(cat PROMPT.md)" --output-format stream-json 2>&1 | tee -a "$LOGFILE"

    echo ""
    echo "--- Iteration #${COUNT} terminée ---"
    echo ""

    # Vérifier si on a atteint le max
    if [[ "$MAX_ITERATIONS" -gt 0 && "$COUNT" -ge "$MAX_ITERATIONS" ]]; then
        echo "=== ${MAX_ITERATIONS} itérations complétées. Arrêt. ==="
        break
    fi

    # Petite pause entre les itérations
    sleep 2
done
