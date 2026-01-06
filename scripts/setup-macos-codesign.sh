#!/bin/bash
set -euo pipefail

# macOS Code Signing Setup Script
# Usage:
#   ./scripts/setup-macos-codesign.sh verify    - Verify certificate is available
#   ./scripts/setup-macos-codesign.sh import    - Import certificate (CI mode)

COMMAND="${1:-verify}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

check_macos() {
    if [[ "$(uname)" != "Darwin" ]]; then
        log_error "This script only runs on macOS"
        exit 1
    fi
}

verify_certificate() {
    log_info "Checking for code signing certificates..."

    IDENTITIES=$(security find-identity -v -p codesigning 2>/dev/null || true)

    if [[ -z "$IDENTITIES" ]]; then
        log_error "No code signing identities found"
        log_info "To set up code signing:"
        log_info "  1. Open Keychain Access"
        log_info "  2. Import your Developer ID Application certificate (.p12)"
        exit 1
    fi

    echo ""
    echo "Available signing identities:"
    echo "$IDENTITIES"
    echo ""

    DEVELOPER_ID=$(echo "$IDENTITIES" | grep "Developer ID Application" || true)

    if [[ -z "$DEVELOPER_ID" ]]; then
        log_warn "No 'Developer ID Application' certificate found"
        log_info "For distribution outside the App Store, you need a Developer ID Application certificate"
        exit 1
    fi

    log_info "Developer ID Application certificate found"

    CERT_NAME=$(echo "$DEVELOPER_ID" | head -1 | sed 's/.*"\(.*\)".*/\1/')
    log_info "Certificate: $CERT_NAME"

    if [[ -n "${APPLE_SIGNING_IDENTITY:-}" ]]; then
        if echo "$IDENTITIES" | grep -q "$APPLE_SIGNING_IDENTITY"; then
            log_info "APPLE_SIGNING_IDENTITY matches available certificate"
        else
            log_warn "APPLE_SIGNING_IDENTITY does not match any available certificate"
            log_info "Expected: $APPLE_SIGNING_IDENTITY"
        fi
    fi

    log_info "Checking notarization credentials..."
    MISSING_VARS=()

    [[ -z "${APPLE_API_KEY:-}" ]] && MISSING_VARS+=("APPLE_API_KEY")
    [[ -z "${APPLE_API_KEY_ID:-}" ]] && MISSING_VARS+=("APPLE_API_KEY_ID")
    [[ -z "${APPLE_API_ISSUER:-}" ]] && MISSING_VARS+=("APPLE_API_ISSUER")

    if [[ ${#MISSING_VARS[@]} -gt 0 ]]; then
        log_warn "Missing notarization environment variables:"
        for var in "${MISSING_VARS[@]}"; do
            echo "  - $var"
        done
        log_info "Notarization will be skipped without these variables"
    else
        log_info "All notarization credentials are set"
    fi

    echo ""
    log_info "Certificate verification complete"
}

import_certificate() {
    log_info "Importing certificate for CI..."

    if [[ -z "${APPLE_CERTIFICATE:-}" ]]; then
        log_error "APPLE_CERTIFICATE environment variable is not set"
        exit 1
    fi

    if [[ -z "${APPLE_CERTIFICATE_PASSWORD:-}" ]]; then
        log_error "APPLE_CERTIFICATE_PASSWORD environment variable is not set"
        exit 1
    fi

    TEMP_DIR="${RUNNER_TEMP:-$(mktemp -d)}"
    CERTIFICATE_PATH="$TEMP_DIR/certificate.p12"
    KEYCHAIN_PATH="$TEMP_DIR/app-signing.keychain-db"
    KEYCHAIN_PASSWORD=$(uuidgen)

    log_info "Decoding certificate..."
    echo -n "$APPLE_CERTIFICATE" | base64 --decode -o "$CERTIFICATE_PATH"

    log_info "Creating temporary keychain..."
    security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
    security set-keychain-settings -lut 21600 "$KEYCHAIN_PATH"
    security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"

    log_info "Importing certificate to keychain..."
    security import "$CERTIFICATE_PATH" \
        -P "$APPLE_CERTIFICATE_PASSWORD" \
        -A \
        -t cert \
        -f pkcs12 \
        -k "$KEYCHAIN_PATH"

    security set-key-partition-list -S apple-tool:,apple: -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
    security list-keychain -d user -s "$KEYCHAIN_PATH"

    rm -f "$CERTIFICATE_PATH"

    log_info "Certificate imported successfully"

    verify_certificate
}

show_usage() {
    echo "macOS Code Signing Setup Script"
    echo ""
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  verify  - Verify that signing certificate is available (default)"
    echo "  import  - Import certificate from environment variables (CI mode)"
    echo ""
    echo "Environment variables for 'import':"
    echo "  APPLE_CERTIFICATE          - Base64-encoded .p12 certificate"
    echo "  APPLE_CERTIFICATE_PASSWORD - Password for the .p12 file"
    echo ""
    echo "Environment variables for notarization (optional for verify):"
    echo "  APPLE_SIGNING_IDENTITY - Certificate name (e.g., 'Developer ID Application: Name (ID)')"
    echo "  APPLE_API_KEY          - App Store Connect API key contents"
    echo "  APPLE_API_KEY_ID       - App Store Connect API key ID"
    echo "  APPLE_API_ISSUER       - App Store Connect Issuer ID"
}

check_macos

case "$COMMAND" in
    verify)
        verify_certificate
        ;;
    import)
        import_certificate
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        log_error "Unknown command: $COMMAND"
        show_usage
        exit 1
        ;;
esac
