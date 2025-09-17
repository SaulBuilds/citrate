#!/bin/bash
# Sprint 5: Comprehensive Security Audit Suite
# Security scanning, vulnerability assessment, and penetration testing

set -e

echo "================================================"
echo "     Lattice V3 Security Audit - Sprint 5"
echo "================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="${PROJECT_ROOT:-$(pwd)}"
RESULTS_DIR="${RESULTS_DIR:-./security-results}"
SCAN_LEVEL="${SCAN_LEVEL:-comprehensive}" # basic, standard, comprehensive

# Security issue counters
CRITICAL_ISSUES=0
HIGH_ISSUES=0
MEDIUM_ISSUES=0
LOW_ISSUES=0
INFO_ISSUES=0

# Create results directory
mkdir -p "$RESULTS_DIR"

# Function to print colored status
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SECURE]${NC} $1"
}

print_critical() {
    echo -e "${RED}[CRITICAL]${NC} $1"
    ((CRITICAL_ISSUES++))
}

print_high() {
    echo -e "${RED}[HIGH]${NC} $1"
    ((HIGH_ISSUES++))
}

print_medium() {
    echo -e "${YELLOW}[MEDIUM]${NC} $1"
    ((MEDIUM_ISSUES++))
}

print_low() {
    echo -e "${YELLOW}[LOW]${NC} $1"
    ((LOW_ISSUES++))
}

print_info() {
    echo -e "${PURPLE}[INFO]${NC} $1"
    ((INFO_ISSUES++))
}

# ============================================
# 1. Dependency Vulnerability Scanning
# ============================================

scan_dependencies() {
    print_status "1. Dependency Vulnerability Scanning"
    
    # Rust dependencies
    print_status "Scanning Rust dependencies with cargo-audit..."
    if command -v cargo-audit &> /dev/null; then
        cd "$PROJECT_ROOT"
        if cargo audit 2>&1 | tee "$RESULTS_DIR/cargo-audit.txt" | grep -q "vulnerabilities found"; then
            VULNS=$(grep -c "^ID:" "$RESULTS_DIR/cargo-audit.txt" || echo "0")
            if [ "$VULNS" -gt 0 ]; then
                print_high "Found $VULNS vulnerabilities in Rust dependencies"
            fi
        else
            print_success "No known vulnerabilities in Rust dependencies"
        fi
    else
        print_info "cargo-audit not installed, skipping Rust dependency scan"
    fi
    
    # Node.js dependencies
    print_status "Scanning Node.js dependencies..."
    if [ -d "$PROJECT_ROOT/gui/lattice-core" ]; then
        cd "$PROJECT_ROOT/gui/lattice-core"
        if command -v npm &> /dev/null; then
            npm audit --json > "$RESULTS_DIR/npm-audit.json" 2>/dev/null || true
            
            CRITICAL=$(jq '.metadata.vulnerabilities.critical // 0' "$RESULTS_DIR/npm-audit.json" 2>/dev/null || echo "0")
            HIGH=$(jq '.metadata.vulnerabilities.high // 0' "$RESULTS_DIR/npm-audit.json" 2>/dev/null || echo "0")
            
            if [ "$CRITICAL" -gt 0 ]; then
                print_critical "Found $CRITICAL critical vulnerabilities in npm dependencies"
            fi
            if [ "$HIGH" -gt 0 ]; then
                print_high "Found $HIGH high vulnerabilities in npm dependencies"
            fi
            
            if [ "$CRITICAL" -eq 0 ] && [ "$HIGH" -eq 0 ]; then
                print_success "No critical/high vulnerabilities in npm dependencies"
            fi
        fi
    fi
    
    cd "$PROJECT_ROOT"
}

# ============================================
# 2. Static Application Security Testing (SAST)
# ============================================

sast_scan() {
    print_status "2. Static Application Security Testing (SAST)"
    
    # Check for hardcoded secrets
    print_status "Scanning for hardcoded secrets..."
    
    # Common patterns for secrets
    SECRET_PATTERNS=(
        "password.*=.*['\"].*['\"]"
        "api[_-]?key.*=.*['\"].*['\"]"
        "secret.*=.*['\"].*['\"]"
        "token.*=.*['\"].*['\"]"
        "private[_-]?key.*=.*['\"].*['\"]"
    )
    
    SECRETS_FOUND=0
    for pattern in "${SECRET_PATTERNS[@]}"; do
        MATCHES=$(grep -r -i "$pattern" --include="*.rs" --include="*.ts" --include="*.js" \
                  --exclude-dir=target --exclude-dir=node_modules "$PROJECT_ROOT" 2>/dev/null | wc -l)
        if [ "$MATCHES" -gt 0 ]; then
            ((SECRETS_FOUND += MATCHES))
        fi
    done
    
    if [ "$SECRETS_FOUND" -gt 0 ]; then
        print_medium "Found $SECRETS_FOUND potential hardcoded secrets"
    else
        print_success "No hardcoded secrets detected"
    fi
    
    # Check for unsafe Rust code
    print_status "Checking for unsafe Rust code..."
    UNSAFE_COUNT=$(grep -r "unsafe" --include="*.rs" "$PROJECT_ROOT/core" "$PROJECT_ROOT/node" 2>/dev/null | wc -l)
    
    if [ "$UNSAFE_COUNT" -gt 0 ]; then
        print_low "Found $UNSAFE_COUNT uses of 'unsafe' in Rust code"
        echo "Unsafe code locations:" >> "$RESULTS_DIR/unsafe-code.txt"
        grep -r -n "unsafe" --include="*.rs" "$PROJECT_ROOT/core" "$PROJECT_ROOT/node" >> "$RESULTS_DIR/unsafe-code.txt" 2>/dev/null
    else
        print_success "No unsafe Rust code detected"
    fi
    
    # Check for SQL injection vulnerabilities
    print_status "Checking for SQL injection risks..."
    SQL_CONCAT=$(grep -r "format!.*SELECT\|UPDATE\|INSERT\|DELETE" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$SQL_CONCAT" -gt 0 ]; then
        print_high "Found $SQL_CONCAT potential SQL injection points"
    else
        print_success "No obvious SQL injection vulnerabilities"
    fi
}

# ============================================
# 3. Cryptographic Implementation Review
# ============================================

crypto_review() {
    print_status "3. Cryptographic Implementation Review"
    
    # Check for weak random number generation
    print_status "Checking random number generation..."
    WEAK_RAND=$(grep -r "rand::random\|rand()" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$WEAK_RAND" -gt 0 ]; then
        print_medium "Found $WEAK_RAND uses of potentially weak random number generation"
    else
        print_success "No weak random number generation detected"
    fi
    
    # Check for proper key derivation
    print_status "Checking key derivation functions..."
    KDF_USAGE=$(grep -r "pbkdf2\|argon2\|scrypt\|bcrypt" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$KDF_USAGE" -gt 0 ]; then
        print_success "Found $KDF_USAGE uses of proper key derivation functions"
    else
        print_info "No standard KDF usage found - verify custom implementation"
    fi
    
    # Check signature verification
    print_status "Checking signature verification..."
    SIG_VERIFY=$(grep -r "verify_signature\|signature.*verify" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$SIG_VERIFY" -gt 0 ]; then
        print_success "Found $SIG_VERIFY signature verification points"
    else
        print_high "No signature verification found - critical for consensus"
    fi
}

# ============================================
# 4. Network Security Assessment
# ============================================

network_security() {
    print_status "4. Network Security Assessment"
    
    # Check for TLS/SSL usage
    print_status "Checking TLS/SSL configuration..."
    TLS_USAGE=$(grep -r "rustls\|openssl\|tls\|https" --include="*.rs" --include="*.toml" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$TLS_USAGE" -gt 0 ]; then
        print_success "TLS/SSL configuration found"
    else
        print_high "No TLS/SSL configuration detected - communications may be unencrypted"
    fi
    
    # Check for rate limiting
    print_status "Checking rate limiting implementation..."
    RATE_LIMIT=$(grep -r "rate.*limit\|throttle\|RateLimiter" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$RATE_LIMIT" -gt 0 ]; then
        print_success "Rate limiting implementation found"
    else
        print_medium "No rate limiting found - vulnerable to DoS attacks"
    fi
    
    # Check for IP filtering
    print_status "Checking IP filtering/firewall rules..."
    IP_FILTER=$(grep -r "ip.*filter\|whitelist\|blacklist\|ban.*list" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$IP_FILTER" -gt 0 ]; then
        print_success "IP filtering mechanisms found"
    else
        print_low "No IP filtering found - consider implementing"
    fi
}

# ============================================
# 5. Smart Contract Security (if applicable)
# ============================================

contract_security() {
    print_status "5. Smart Contract Security Analysis"
    
    if [ -d "$PROJECT_ROOT/contracts" ]; then
        print_status "Analyzing Solidity contracts..."
        
        # Check for reentrancy vulnerabilities
        REENTRANCY=$(grep -r "call\|send\|transfer" --include="*.sol" "$PROJECT_ROOT/contracts" 2>/dev/null | wc -l)
        
        if [ "$REENTRANCY" -gt 0 ]; then
            print_medium "Found $REENTRANCY external calls - check for reentrancy guards"
        fi
        
        # Check for overflow protection
        SAFE_MATH=$(grep -r "SafeMath\|checked" --include="*.sol" "$PROJECT_ROOT/contracts" 2>/dev/null | wc -l)
        
        if [ "$SAFE_MATH" -gt 0 ]; then
            print_success "Overflow protection found"
        else
            print_high "No overflow protection detected in contracts"
        fi
        
        # Run Slither if available
        if command -v slither &> /dev/null; then
            print_status "Running Slither security analyzer..."
            slither "$PROJECT_ROOT/contracts" --json "$RESULTS_DIR/slither-report.json" 2>/dev/null || true
            print_info "Slither analysis complete - check report for details"
        fi
    else
        print_info "No contracts directory found - skipping contract security"
    fi
}

# ============================================
# 6. Access Control & Authentication
# ============================================

access_control() {
    print_status "6. Access Control & Authentication Review"
    
    # Check for authentication mechanisms
    print_status "Checking authentication implementation..."
    AUTH_IMPL=$(grep -r "authenticate\|authorization\|jwt\|session" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$AUTH_IMPL" -gt 0 ]; then
        print_success "Authentication mechanisms found"
    else
        print_info "Limited authentication found - verify if needed"
    fi
    
    # Check for role-based access control
    print_status "Checking access control..."
    RBAC=$(grep -r "role\|permission\|admin\|validator" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$RBAC" -gt 0 ]; then
        print_success "Role-based access control patterns found"
    else
        print_low "No RBAC patterns found - consider implementing"
    fi
}

# ============================================
# 7. Input Validation & Sanitization
# ============================================

input_validation() {
    print_status "7. Input Validation & Sanitization"
    
    # Check for input validation
    print_status "Checking input validation..."
    VALIDATION=$(grep -r "validate\|sanitize\|check.*input\|verify.*input" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$VALIDATION" -gt 20 ]; then
        print_success "Extensive input validation found ($VALIDATION occurrences)"
    elif [ "$VALIDATION" -gt 0 ]; then
        print_medium "Limited input validation found ($VALIDATION occurrences)"
    else
        print_high "No input validation found - critical vulnerability"
    fi
    
    # Check for bounds checking
    print_status "Checking bounds validation..."
    BOUNDS=$(grep -r "check.*bounds\|check.*range\|check.*limit" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$BOUNDS" -gt 0 ]; then
        print_success "Bounds checking found"
    else
        print_medium "Limited bounds checking - potential overflow risk"
    fi
}

# ============================================
# 8. Error Handling & Information Disclosure
# ============================================

error_handling() {
    print_status "8. Error Handling & Information Disclosure"
    
    # Check for sensitive info in errors
    print_status "Checking error message content..."
    SENSITIVE_ERRORS=$(grep -r "panic!\|unwrap()\|expect(" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$SENSITIVE_ERRORS" -gt 50 ]; then
        print_medium "Found $SENSITIVE_ERRORS potential panic points - may leak information"
    elif [ "$SENSITIVE_ERRORS" -gt 0 ]; then
        print_low "Found $SENSITIVE_ERRORS panic/unwrap uses"
    else
        print_success "Minimal use of panic/unwrap"
    fi
    
    # Check for debug information
    print_status "Checking for debug information in release..."
    DEBUG_INFO=$(grep -r "println!\|dbg!\|eprintln!" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$DEBUG_INFO" -gt 20 ]; then
        print_low "Found $DEBUG_INFO debug print statements"
    else
        print_success "Minimal debug output in code"
    fi
}

# ============================================
# 9. Consensus Security
# ============================================

consensus_security() {
    print_status "9. Consensus Security Analysis"
    
    # Check for double-spend protection
    print_status "Checking double-spend protection..."
    DOUBLE_SPEND=$(grep -r "double.*spend\|nonce.*check\|replay.*protect" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$DOUBLE_SPEND" -gt 0 ]; then
        print_success "Double-spend protection mechanisms found"
    else
        print_critical "No explicit double-spend protection found"
    fi
    
    # Check for finality mechanisms
    print_status "Checking finality implementation..."
    FINALITY=$(grep -r "finality\|finalize\|checkpoint" --include="*.rs" "$PROJECT_ROOT" 2>/dev/null | wc -l)
    
    if [ "$FINALITY" -gt 0 ]; then
        print_success "Finality mechanisms implemented"
    else
        print_high "No finality mechanisms found - chain may be vulnerable to reorgs"
    fi
}

# ============================================
# 10. Generate Security Report
# ============================================

generate_security_report() {
    local total_issues=$((CRITICAL_ISSUES + HIGH_ISSUES + MEDIUM_ISSUES + LOW_ISSUES))
    
    cat > "$RESULTS_DIR/security-audit-report.json" <<EOF
{
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "scan_level": "$SCAN_LEVEL",
    "project_root": "$PROJECT_ROOT",
    "summary": {
        "total_issues": $total_issues,
        "critical": $CRITICAL_ISSUES,
        "high": $HIGH_ISSUES,
        "medium": $MEDIUM_ISSUES,
        "low": $LOW_ISSUES,
        "info": $INFO_ISSUES
    },
    "categories_scanned": [
        "dependency_vulnerabilities",
        "static_analysis",
        "cryptographic_implementation",
        "network_security",
        "smart_contract_security",
        "access_control",
        "input_validation",
        "error_handling",
        "consensus_security"
    ],
    "risk_level": "$(calculate_risk_level)",
    "recommendations": [
        "Address all critical and high severity issues immediately",
        "Review and fix medium severity issues before production",
        "Consider low severity issues for future improvements",
        "Implement continuous security scanning in CI/CD pipeline",
        "Conduct regular security audits"
    ]
}
EOF
    
    print_status "Security audit report saved to $RESULTS_DIR/security-audit-report.json"
}

calculate_risk_level() {
    if [ "$CRITICAL_ISSUES" -gt 0 ]; then
        echo "CRITICAL"
    elif [ "$HIGH_ISSUES" -gt 2 ]; then
        echo "HIGH"
    elif [ "$MEDIUM_ISSUES" -gt 5 ]; then
        echo "MEDIUM"
    elif [ "$LOW_ISSUES" -gt 10 ]; then
        echo "LOW"
    else
        echo "MINIMAL"
    fi
}

# ============================================
# Main Execution
# ============================================

main() {
    echo ""
    print_status "Starting Security Audit"
    print_status "Project Root: $PROJECT_ROOT"
    print_status "Scan Level: $SCAN_LEVEL"
    echo ""
    
    # Run security scans
    scan_dependencies
    echo ""
    
    sast_scan
    echo ""
    
    crypto_review
    echo ""
    
    network_security
    echo ""
    
    contract_security
    echo ""
    
    access_control
    echo ""
    
    input_validation
    echo ""
    
    error_handling
    echo ""
    
    consensus_security
    echo ""
    
    # Generate report
    generate_security_report
    
    echo ""
    echo "================================================"
    echo "           Security Audit Summary"
    echo "================================================"
    echo ""
    echo -e "Critical Issues: ${RED}$CRITICAL_ISSUES${NC}"
    echo -e "High Issues:     ${RED}$HIGH_ISSUES${NC}"
    echo -e "Medium Issues:   ${YELLOW}$MEDIUM_ISSUES${NC}"
    echo -e "Low Issues:      ${YELLOW}$LOW_ISSUES${NC}"
    echo -e "Info Items:      ${PURPLE}$INFO_ISSUES${NC}"
    echo ""
    
    local risk_level=$(calculate_risk_level)
    echo -e "Overall Risk Level: $(color_risk_level $risk_level)"
    echo ""
    
    if [ "$CRITICAL_ISSUES" -gt 0 ] || [ "$HIGH_ISSUES" -gt 0 ]; then
        print_status "⚠️  Critical/High issues found - immediate action required"
        exit 1
    else
        print_success "✅ No critical security issues found"
        exit 0
    fi
}

color_risk_level() {
    case $1 in
        CRITICAL) echo -e "${RED}CRITICAL${NC}" ;;
        HIGH) echo -e "${RED}HIGH${NC}" ;;
        MEDIUM) echo -e "${YELLOW}MEDIUM${NC}" ;;
        LOW) echo -e "${YELLOW}LOW${NC}" ;;
        MINIMAL) echo -e "${GREEN}MINIMAL${NC}" ;;
        *) echo "$1" ;;
    esac
}

# Run main function
main