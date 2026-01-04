#!/bin/bash
# Pretty printer Stop hook - blocks Claude from stopping until all tests pass
# Only runs between 6pm and 4am (stops during daytime hours)

# Log to file for debugging (check /tmp/pp-hook.log)
LOG="/tmp/pp-hook.log"
echo "$(date): Hook triggered" >> "$LOG"

cd "$CLAUDE_PROJECT_DIR" 2>/dev/null || cd "$(dirname "$0")/.."

# Read hook input from stdin
input=$(cat)

# Check if we're already in a stop hook loop to prevent infinite recursion
stop_hook_active=$(echo "$input" | grep -o '"stop_hook_active":[^,}]*' | cut -d: -f2 | tr -d ' ')

if [ "$stop_hook_active" = "true" ]; then
    echo "$(date): Stop hook active, allowing exit" >> "$LOG"
    exit 0
fi

# Check time - only run between 6pm (18:00) and 4am (04:00)
HOUR=$(date +%H)
if [ "$HOUR" -ge 4 ] && [ "$HOUR" -lt 18 ]; then
    echo "$(date): Outside allowed hours (hour=$HOUR), allowing exit" >> "$LOG"
    exit 0
fi

# Run tests and capture result
TEST_OUTPUT=$(cargo test -p pgls_pretty_print --no-fail-fast 2>&1)
RESULT_LINE=$(echo "$TEST_OUTPUT" | grep "^test result:" | grep -v "0 passed; 0 failed" | tail -1)

PASSED=$(echo "$RESULT_LINE" | grep -oE "[0-9]+ passed" | grep -oE "[0-9]+" || echo "0")
FAILED=$(echo "$RESULT_LINE" | grep -oE "[0-9]+ failed" | grep -oE "[0-9]+" || echo "0")

echo "$(date): Tests - $PASSED passed, $FAILED failed" >> "$LOG"

# Check if all tests pass
if [ "$FAILED" = "0" ] && [ "$PASSED" -gt 0 ]; then
    echo "$(date): All tests passing, allowing exit" >> "$LOG"
    exit 0
else
    # Block Claude from stopping - more work needed
    echo "$(date): Tests failing, blocking exit" >> "$LOG"
    # Output ONLY the JSON decision - no other stdout
    echo '{"decision":"block","reason":"Tests still failing. Continue fixing node implementations."}'
    exit 0
fi
