#!/bin/bash

# Pump Sniper Installation Script
# This script downloads and installs Pump Sniper binaries

set -e

REPO="kernlog/pump-sniper"
VERSION="v0.1.0-alpha"
INSTALL_DIR="/usr/local/bin"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    case "$OS" in
        darwin)
            PLATFORM="darwin"
            ;;
        linux)
            PLATFORM="linux"
            ;;
        *)
            echo -e "${RED}Unsupported operating system: $OS${NC}"
            exit 1
            ;;
    esac
    
    case "$ARCH" in
        x86_64)
            ARCH="amd64"
            ;;
        arm64|aarch64)
            ARCH="arm64"
            ;;
        *)
            echo -e "${RED}Unsupported architecture: $ARCH${NC}"
            exit 1
            ;;
    esac
    
    BINARY_NAME="pump-sniper-${VERSION}-${PLATFORM}-${ARCH}"
}

# Download and install binaries
install_binaries() {
    echo -e "${GREEN}Installing Pump Sniper ${VERSION}...${NC}"
    
    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Download release
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}.tar.gz"
    echo "Downloading from: $DOWNLOAD_URL"
    
    if ! curl -L -o "${BINARY_NAME}.tar.gz" "$DOWNLOAD_URL"; then
        echo -e "${RED}Failed to download release. The binary for your platform might not be available yet.${NC}"
        echo -e "${YELLOW}You can build from source instead:${NC}"
        echo "  git clone https://github.com/${REPO}.git"
        echo "  cd pump-sniper"
        echo "  cargo build --release"
        rm -rf "$TEMP_DIR"
        exit 1
    fi
    
    # Extract archive
    tar -xzf "${BINARY_NAME}.tar.gz"
    
    # Check if user has write permissions to install directory
    if [ -w "$INSTALL_DIR" ]; then
        mv monitor "$INSTALL_DIR/"
        mv sniper "$INSTALL_DIR/"
    else
        echo -e "${YELLOW}Need sudo permissions to install to $INSTALL_DIR${NC}"
        sudo mv monitor "$INSTALL_DIR/"
        sudo mv sniper "$INSTALL_DIR/"
    fi
    
    # Clean up
    cd - > /dev/null
    rm -rf "$TEMP_DIR"
    
    echo -e "${GREEN}Installation complete!${NC}"
    echo ""
    echo "Binaries installed to: $INSTALL_DIR"
    echo "  - monitor: Token monitoring tool"
    echo "  - sniper: Automated trading bot"
    echo ""
    echo "To get started:"
    echo "  1. Set up your environment variables (see README.md)"
    echo "  2. Run 'monitor' for monitoring mode"
    echo "  3. Run 'sniper' for trading mode (requires wallet key)"
}

# Check for required tools
check_requirements() {
    if ! command -v curl &> /dev/null; then
        echo -e "${RED}curl is required but not installed.${NC}"
        exit 1
    fi
    
    if ! command -v tar &> /dev/null; then
        echo -e "${RED}tar is required but not installed.${NC}"
        exit 1
    fi
}

# Main installation flow
main() {
    echo "======================================"
    echo "   Pump Sniper Installation Script"
    echo "======================================"
    echo ""
    
    check_requirements
    detect_platform
    
    echo "Detected platform: ${PLATFORM}-${ARCH}"
    echo ""
    
    install_binaries
    
    # Verify installation
    if command -v monitor &> /dev/null && command -v sniper &> /dev/null; then
        echo -e "${GREEN}Verification successful! Both binaries are available in your PATH.${NC}"
    else
        echo -e "${YELLOW}Note: You may need to add $INSTALL_DIR to your PATH${NC}"
        echo "Add this to your shell profile:"
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
    fi
}

# Allow custom version installation
if [ -n "$1" ]; then
    VERSION="$1"
fi

main