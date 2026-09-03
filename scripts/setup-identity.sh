#!/usr/bin/env bash
set -e

IDENTITY_NAME="${1:-deployer}"
NETWORK="${2:-testnet}"

echo "============================================"
echo " Stellar Primitives Identity Setup Helpers "
echo "============================================"

# Ensure Stellar CLI is installed
if ! command -v stellar &> /dev/null; then
    echo "Error: stellar CLI is not installed."
    echo "Install via: cargo install --locked stellar-cli --features opt"
    exit 1
fi

echo "Configuring network: ${NETWORK}..."
stellar network add --global ${NETWORK} \
    --rpc-url https://soroban-testnet.stellar.org:443 \
    --network-passphrase "Test SDF Network ; September 2015" || true

echo "Generating identity '${IDENTITY_NAME}'..."
stellar keys generate ${IDENTITY_NAME} --global || true

echo "Funding identity '${IDENTITY_NAME}' via Friendbot..."
stellar keys fund ${IDENTITY_NAME} --network ${NETWORK} || true

echo ""
echo "Identity setup completed successfully!"
echo "Identity Address: $(stellar keys address ${IDENTITY_NAME})"
