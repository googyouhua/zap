#!/bin/bash
# Warp bootstrap debug script
OUTFILE="/tmp/warp_debug_$$.txt"

{
echo "=== Warp Bootstrap Debug ==="
echo "Timestamp: $(date)"
echo ""

echo "=== 1. SSH_CLIENT ==="
echo "SSH_CLIENT=[${SSH_CLIENT-(unset)}]"
echo "SSH_CLIENT length=${#SSH_CLIENT}"

echo ""
echo "=== 2. WARP_BOOTSTRAPPED ==="
echo "WARP_BOOTSTRAPPED=[${WARP_BOOTSTRAPPED-(unset)}]"

echo ""
echo "=== 3. WARP_USING_WINDOWS_CON_PTY ==="
echo "WARP_USING_WINDOWS_CON_PTY=[${WARP_USING_WINDOWS_CON_PTY-(unset)}]"

echo ""
echo "=== 4. Simulate SSH_CLIENT check ==="
if [ -z "$SSH_CLIENT" ]; then
    echo "SSH_CLIENT IS EMPTY -> would INCLUDE compgen fields"
else
    echo "SSH_CLIENT HAS VALUE -> would SKIP compgen fields"
fi

echo ""
echo "=== 5. PATH length ==="
echo "PATH length: ${#PATH}"
echo "PATH truncated (256): [${PATH:0:256}]..."

echo ""
echo "=== 6. compgen -c (all executables) ==="
echo "compgen -c count: $(compgen -c 2>/dev/null | wc -l)"
echo "compgen -c sample (first 5):"
compgen -c 2>/dev/null | head -5

echo ""
echo "=== 7. compgen -e (env var names) ==="
echo "compgen -e count: $(compgen -e 2>/dev/null | wc -l)"
echo "compgen -e sample (first 5):"
compgen -e 2>/dev/null | head -5

echo ""
echo "=== 8. compgen -A function (function names) ==="
echo "compgen -A function count: $(compgen -A function 2>/dev/null | wc -l)"
echo "compgen -A function sample (first 10):"
compgen -A function 2>/dev/null | head -10

echo ""
echo "=== 9. compgen -b (builtins) ==="
echo "compgen -b count: $(compgen -b 2>/dev/null | wc -l)"
echo "compgen -b sample (first 5):"
compgen -b 2>/dev/null | head -5

echo ""
echo "=== 10. compgen -k (keywords) ==="
echo "compgen -k count: $(compgen -k 2>/dev/null | wc -l)"
compgen -k 2>/dev/null

echo ""
echo "=== 11. alias ==="
alias 2>/dev/null | head -10
echo "alias count: $(alias 2>/dev/null | wc -l)"

echo ""
echo "=== 12. Shell options (shopt -s) ==="
shopt -s 2>/dev/null | head -10
echo "shopt -s count: $(shopt -s 2>/dev/null | wc -l)"

echo ""
echo "=== Done ==="
} > "$OUTFILE" 2>&1

echo "Debug output written to: $OUTFILE"
echo "Run: cat $OUTFILE"
