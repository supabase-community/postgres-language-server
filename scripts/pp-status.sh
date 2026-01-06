#!/bin/bash
# Pretty printer implementation status
# Note: Don't use set -e because cargo test exits non-zero on failures
cd "$(dirname "$0")/.."

echo "=== Pretty Printer Status ==="
echo ""

# Run tests and capture the result line (find the one with actual results)
TEST_OUTPUT=$(cargo test -p pgls_pretty_print --no-fail-fast 2>&1)
RESULT_LINE=$(echo "$TEST_OUTPUT" | grep "^test result:" | grep -v "0 passed; 0 failed" | tail -1)

# Fallback if no non-zero results
if [ -z "$RESULT_LINE" ]; then
    RESULT_LINE=$(echo "$TEST_OUTPUT" | grep "^test result:" | tail -1)
fi

# Parse passed/failed from "test result: FAILED. 197 passed; 250 failed; ..."
PASSED=$(echo "$RESULT_LINE" | grep -oE "[0-9]+ passed" | grep -oE "[0-9]+" || echo "0")
FAILED=$(echo "$RESULT_LINE" | grep -oE "[0-9]+ failed" | grep -oE "[0-9]+" || echo "0")
TOTAL=$((PASSED + FAILED))
if [ $TOTAL -gt 0 ]; then
    PCT=$(echo "scale=1; $PASSED * 100 / $TOTAL" | bc)
else
    PCT="0"
fi

echo "Tests: $PASSED/$TOTAL passed ($PCT%)"

# Count implemented nodes from pretty_printer.md
IMPLEMENTED=$(grep -c "^\- \[x\]" agentic/pretty_printer.md 2>/dev/null || echo "0")
echo "Nodes: $IMPLEMENTED/270 documented"
echo ""

# Find partial implementations (files with TODO)
TODO_FILES=$(grep -l "TODO" crates/pgls_pretty_print/src/nodes/*.rs 2>/dev/null | wc -l | tr -d ' ')
if [ "$TODO_FILES" -gt 0 ]; then
    echo "Files with TODOs: $TODO_FILES"
    grep -l "TODO" crates/pgls_pretty_print/src/nodes/*.rs 2>/dev/null | while read f; do
        FILE=$(basename "$f" .rs)
        COUNT=$(grep -c "TODO" "$f")
        echo "  - $FILE: $COUNT"
    done | head -10
    echo ""
fi

# Show pending snapshots if any
PENDING=$(find crates/pgls_pretty_print/tests/snapshots -name "*.snap.new" 2>/dev/null | wc -l | tr -d ' ')
if [ "$PENDING" -gt 0 ]; then
    echo "Pending snapshots: $PENDING (run 'just pp-review')"
    echo ""
fi

echo "Commands:"
echo "  just pp-test <pattern>  - Test matching files"
echo "  just pp-failing         - Show failing tests"
echo "  just pp-review          - Review snapshots"
