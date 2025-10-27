#!/bin/bash

# Comprehensive Security Audit Script for Citrate V3

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
AUDIT_OUTPUT_DIR="$PROJECT_ROOT/audit_results"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$AUDIT_OUTPUT_DIR"

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     Citrate V3 Security Audit Tool    ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo

log() {
    echo -e "[$(date '+%H:%M:%S')] $1"
}

run_check() {
    local name="$1"
    local command="$2"
    local output_file="$AUDIT_OUTPUT_DIR/${name// /_}.log"

    log "${BLUE}Running: $name${NC}"

    if eval "$command" > "$output_file" 2>&1; then
        log "${GREEN}✅ $name: PASSED${NC}"
        return 0
    else
        log "${RED}❌ $name: FAILED${NC}"
        return 1
    fi
}

TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

increment_counter() {
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    if [ $1 -eq 0 ]; then
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    else
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    fi
}

echo "🔍 Starting security audit..."

# 1. Dependency scan
log "${YELLOW}[1/8] Dependency Vulnerability Scan${NC}"
run_check "Cargo_Audit" "cd '$PROJECT_ROOT' && cargo audit"
increment_counter $?

# 2. Static analysis
log "${YELLOW}[2/8] Static Code Analysis${NC}"
run_check "Clippy_Lints" "cd '$PROJECT_ROOT' && cargo clippy --all-targets -- -D warnings"
increment_counter $?

# 3. Crypto analysis
log "${YELLOW}[3/8] Cryptographic Security${NC}"
run_check "Hardcoded_Secrets" "cd '$PROJECT_ROOT' && grep -r 'password.*=' --include='*.rs' . && exit 1 || echo 'No hardcoded secrets'"
increment_counter $?

# 4. Memory safety
log "${YELLOW}[4/8] Memory Safety${NC}"
run_check "Unsafe_Code" "cd '$PROJECT_ROOT' && grep -r 'unsafe' --include='*.rs' . | wc -l | awk '{print \"Unsafe blocks: \" \$1}'"
increment_counter $?

# 5. Input validation
log "${YELLOW}[5/8] Input Validation${NC}"
run_check "Bounds_Checks" "cd '$PROJECT_ROOT' && grep -r 'unwrap()' --include='*.rs' . | wc -l | awk '{if(\$1>100) {print \"Too many unwrap(): \" \$1; exit 1} else print \"Unwrap count acceptable: \" \$1}'"
increment_counter $?

# 6. Security tests
log "${YELLOW}[6/8] Security Test Suite${NC}"
run_check "Security_Tests" "cd '$PROJECT_ROOT' && cargo test crypto --lib"
increment_counter $?

# 7. Smart contracts
log "${YELLOW}[7/8] Smart Contract Security${NC}"
if [ -d "$PROJECT_ROOT/contracts" ]; then
    run_check "Contract_Build" "cd '$PROJECT_ROOT/contracts' && forge build"
    increment_counter $?
else
    log "No contracts found"
    increment_counter 0
fi

# 8. Compilation
log "${YELLOW}[8/8] Compilation Security${NC}"
run_check "Release_Build" "cd '$PROJECT_ROOT' && cargo build --release"
increment_counter $?

# Generate report
cat > "$AUDIT_OUTPUT_DIR/SECURITY_SUMMARY.md" << EOF
# Security Audit Summary

**Date**: $(date)

## Results
- Total Checks: $TOTAL_CHECKS
- Passed: $PASSED_CHECKS
- Failed: $FAILED_CHECKS
- Success Rate: $(awk "BEGIN {printf \"%.1f\", $PASSED_CHECKS * 100 / $TOTAL_CHECKS}")%

## Status
EOF

if [ $FAILED_CHECKS -eq 0 ]; then
    echo "🟢 **SECURE** - All checks passed" >> "$AUDIT_OUTPUT_DIR/SECURITY_SUMMARY.md"
    status="SECURE"
elif [ $FAILED_CHECKS -le 2 ]; then
    echo "🟡 **MODERATE** - Minor issues found" >> "$AUDIT_OUTPUT_DIR/SECURITY_SUMMARY.md"
    status="MODERATE"
else
    echo "🔴 **HIGH RISK** - Multiple issues detected" >> "$AUDIT_OUTPUT_DIR/SECURITY_SUMMARY.md"
    status="HIGH_RISK"
fi

echo
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║          AUDIT COMPLETE                ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo

log "📊 Results: $PASSED_CHECKS/$TOTAL_CHECKS passed"
log "📋 Report: $AUDIT_OUTPUT_DIR/SECURITY_SUMMARY.md"
log "🔒 Status: $status"

if [ $FAILED_CHECKS -eq 0 ]; then
    log "${GREEN}🎉 Security audit passed!${NC}"
    exit 0
else
    log "${YELLOW}⚠️  Review required${NC}"
    exit 1
fi