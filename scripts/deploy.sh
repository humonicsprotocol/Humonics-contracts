#!/bin/bash

set -e

NETWORK=""
CONTRACT=""

usage() {
    echo "Usage: $0 <network> <contract>"
    echo "  network: testnet|mainnet"
    echo "  contract: certificate_registry|verification_gateway|hum_token"
    exit 1
}

if [ $# -ne 2 ]; then
    usage
fi

NETWORK=$1
CONTRACT=$2

if [ "$NETWORK" != "testnet" ] && [ "$NETWORK" != "mainnet" ]; then
    echo "Error: Network must be 'testnet' or 'mainnet'"
    usage
fi

if [ "$CONTRACT" != "certificate_registry" ] && [ "$CONTRACT" != "verification_gateway" ] && [ "$CONTRACT" != "hum_token" ]; then
    echo "Error: Contract must be one of: certificate_registry, verification_gateway, hum_token"
    usage
fi

echo "Deploying $CONTRACT to $NETWORK..."

case $NETWORK in
    testnet)
        NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
        RPC_URL="https://soroban-testnet.stellar.org:443"
        SOURCE_ACCOUNT_SECRET="$SOROBAN_TESTNET_SECRET"
        ;;
    mainnet)
        NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
        RPC_URL="https://soroban.stellar.org:443"
        SOURCE_ACCOUNT_SECRET="$SOROBAN_MAINNET_SECRET"
        ;;
esac

if [ -z "$SOURCE_ACCOUNT_SECRET" ]; then
    echo "Error: Source account secret not set in environment variables"
    echo "Set SOROBAN_TESTNET_SECRET for testnet or SOROBAN_MAINNET_SECRET for mainnet"
    exit 1
fi

echo "Building contract..."
cargo build --target wasm32-unknown-unknown --release -p "$CONTRACT"

WASM_FILE="target/wasm32-unknown-unknown/release/$CONTRACT.wasm"

if [ ! -f "$WASM_FILE" ]; then
    echo "Error: WASM file not found at $WASM_FILE"
    exit 1
fi

echo "Deploying contract to $NETWORK..."

CONTRACT_ID=$(soroban contract deploy \
    --wasm "$WASM_FILE" \
    --source "$SOURCE_ACCOUNT_SECRET" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE")

echo "Contract deployed with ID: $CONTRACT_ID"

ENV_FILE=".env.$NETWORK"
echo "CONTRACT_ID_$CONTRACT=$CONTRACT_ID" >> "$ENV_FILE"
echo "RPC_URL_$NETWORK=$RPC_URL" >> "$ENV_FILE"
echo "NETWORK_PASSPHRASE_$NETWORK=$NETWORK_PASSPHRASE" >> "$ENV_FILE"

echo "Contract ID saved to $ENV_FILE"
echo "Deployment complete!"
