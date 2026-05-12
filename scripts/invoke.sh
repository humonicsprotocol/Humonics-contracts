#!/bin/bash

set -e

NETWORK=""
CONTRACT=""
METHOD=""
ARGS=""

usage() {
    echo "Usage: $0 <network> <contract> <method> [args...]"
    echo "  network: testnet|mainnet"
    echo "  contract: certificate_registry|verification_gateway|hum_token"
    echo "  method: contract method name"
    echo "  args: method arguments (space-separated)"
    echo ""
    echo "Examples:"
    echo "  $0 testnet certificate_registry initialize GABCDEF123456789 GXYZ987654321"
    echo "  $0 testnet hum_token initialize GABCDEF123456789 1000000000"
    echo "  $0 testnet verification_gateway initialize GABCDEF123456789"
    exit 1
}

if [ $# -lt 3 ]; then
    usage
fi

NETWORK=$1
CONTRACT=$2
METHOD=$3
shift 3
ARGS=("$@")

if [ "$NETWORK" != "testnet" ] && [ "$NETWORK" != "mainnet" ]; then
    echo "Error: Network must be 'testnet' or 'mainnet'"
    usage
fi

if [ "$CONTRACT" != "certificate_registry" ] && [ "$CONTRACT" != "verification_gateway" ] && [ "$CONTRACT" != "hum_token" ]; then
    echo "Error: Contract must be one of: certificate_registry, verification_gateway, hum_token"
    usage
fi

ENV_FILE=".env.$NETWORK"

if [ ! -f "$ENV_FILE" ]; then
    echo "Error: Environment file $ENV_FILE not found"
    echo "Run deploy.sh first to create the environment file"
    exit 1
fi

source "$ENV_FILE"

CONTRACT_ID_VAR="CONTRACT_ID_$CONTRACT"
CONTRACT_ID="${!CONTRACT_ID_VAR}"

if [ -z "$CONTRACT_ID" ]; then
    echo "Error: Contract ID not found in $ENV_FILE"
    echo "Variable $CONTRACT_ID_VAR is not set"
    exit 1
fi

RPC_URL_VAR="RPC_URL_$NETWORK"
RPC_URL="${!RPC_URL_VAR}"

NETWORK_PASSPHRASE_VAR="NETWORK_PASSPHRASE_$NETWORK"
NETWORK_PASSPHRASE="${!NETWORK_PASSPHRASE_VAR}"

SOURCE_ACCOUNT_SECRET_VAR="SOROBAN_${NETWORK^^}_SECRET"
SOURCE_ACCOUNT_SECRET="${!SOURCE_ACCOUNT_SECRET_VAR}"

if [ -z "$SOURCE_ACCOUNT_SECRET" ]; then
    echo "Error: Source account secret not set in environment variables"
    echo "Set SOROBAN_TESTNET_SECRET for testnet or SOROBAN_MAINNET_SECRET for mainnet"
    exit 1
fi

echo "Invoking $METHOD on $CONTRACT ($CONTRACT_ID) on $NETWORK..."

case $METHOD in
    "initialize")
        case $CONTRACT in
            "certificate_registry")
                if [ ${#ARGS[@]} -ne 2 ]; then
                    echo "Error: initialize for certificate_registry requires 2 arguments: governance_address, proof_verifier_address"
                    exit 1
                fi
                soroban contract invoke \
                    --id "$CONTRACT_ID" \
                    --source "$SOURCE_ACCOUNT_SECRET" \
                    --rpc-url "$RPC_URL" \
                    --network-passphrase "$NETWORK_PASSPHRASE" \
                    -- \
                    initialize \
                    --governance-address "${ARGS[0]}" \
                    --proof-verifier-address "${ARGS[1]}"
                ;;
            "verification_gateway")
                if [ ${#ARGS[@]} -ne 1 ]; then
                    echo "Error: initialize for verification_gateway requires 1 argument: certificate_registry_address"
                    exit 1
                fi
                soroban contract invoke \
                    --id "$CONTRACT_ID" \
                    --source "$SOURCE_ACCOUNT_SECRET" \
                    --rpc-url "$RPC_URL" \
                    --network-passphrase "$NETWORK_PASSPHRASE" \
                    -- \
                    initialize \
                    --certificate-registry-address "${ARGS[0]}"
                ;;
            "hum_token")
                if [ ${#ARGS[@]} -ne 2 ]; then
                    echo "Error: initialize for hum_token requires 2 arguments: admin_address, initial_supply"
                    exit 1
                fi
                soroban contract invoke \
                    --id "$CONTRACT_ID" \
                    --source "$SOURCE_ACCOUNT_SECRET" \
                    --rpc-url "$RPC_URL" \
                    --network-passphrase "$NETWORK_PASSPHRASE" \
                    -- \
                    initialize \
                    --admin "${ARGS[0]}" \
                    --initial-supply "${ARGS[1]}"
                ;;
        esac
        ;;
    "issue_certificate")
        if [ ${#ARGS[@]} -lt 5 ]; then
            echo "Error: issue_certificate requires at least 5 arguments: content_hash, zk_proof_file, human_commitment, content_hash_signal, timestamp, did, content_type"
            exit 1
        fi
        
        if [ ! -f "${ARGS[1]}" ]; then
            echo "Error: ZK proof file ${ARGS[1]} not found"
            exit 1
        fi
        
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            issue_certificate \
            --content-hash "${ARGS[0]}" \
            --zk-proof @"${ARGS[1]}" \
            --public-signals "[\"${ARGS[2]}\", \"${ARGS[3]}\", \"${ARGS[4]}\"]" \
            --did "${ARGS[5]}" \
            --content-type "${ARGS[6]}"
        ;;
    "revoke_certificate")
        if [ ${#ARGS[@]} -ne 2 ]; then
            echo "Error: revoke_certificate requires 2 arguments: cert_id, reason"
            exit 1
        fi
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            revoke_certificate \
            --cert-id "${ARGS[0]}" \
            --reason "${ARGS[1]}"
        ;;
    "get_certificate")
        if [ ${#ARGS[@]} -ne 1 ]; then
            echo "Error: get_certificate requires 1 argument: cert_id"
            exit 1
        fi
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            get_certificate \
            --cert-id "${ARGS[0]}"
        ;;
    "is_certified")
        if [ ${#ARGS[@]} -ne 1 ]; then
            echo "Error: is_certified requires 1 argument: content_hash"
            exit 1
        fi
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            is_certified \
            --content-hash "${ARGS[0]}"
        ;;
    "verify")
        if [ ${#ARGS[@]} -ne 1 ]; then
            echo "Error: verify requires 1 argument: content_hash"
            exit 1
        fi
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            verify \
            --content-hash "${ARGS[0]}"
        ;;
    "batch_verify")
        if [ ${#ARGS[@]} -eq 0 ]; then
            echo "Error: batch_verify requires at least 1 argument: content_hash1 [content_hash2 ...]"
            exit 1
        fi
        
        # Convert args to JSON array format
        CONTENT_HASHES="["
        for i in "${!ARGS[@]}"; do
            if [ $i -gt 0 ]; then
                CONTENT_HASHES+=","
            fi
            CONTENT_HASHES+="\"${ARGS[i]}\""
        done
        CONTENT_HASHES+="]"
        
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            batch_verify \
            --content-hashes "$CONTENT_HASHES"
        ;;
    "stake")
        if [ ${#ARGS[@]} -ne 2 ]; then
            echo "Error: stake requires 2 arguments: amount, staker_address"
            exit 1
        fi
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            stake \
            --amount "${ARGS[0]}" \
            --staker "${ARGS[1]}"
        ;;
    "unstake")
        if [ ${#ARGS[@]} -ne 2 ]; then
            echo "Error: unstake requires 2 arguments: amount, staker_address"
            exit 1
        fi
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            unstake \
            --amount "${ARGS[0]}" \
            --staker "${ARGS[1]}"
        ;;
    "slash")
        if [ ${#ARGS[@]} -ne 3 ]; then
            echo "Error: slash requires 3 arguments: staker_address, amount, reason"
            exit 1
        fi
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            slash \
            --staker "${ARGS[0]}" \
            --amount "${ARGS[1]}" \
            --reason "${ARGS[2]}"
        ;;
    "get_stake")
        if [ ${#ARGS[@]} -ne 1 ]; then
            echo "Error: get_stake requires 1 argument: staker_address"
            exit 1
        fi
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            get_stake \
            --staker "${ARGS[0]}"
        ;;
    "transfer")
        if [ ${#ARGS[@]} -ne 3 ]; then
            echo "Error: transfer requires 3 arguments: from_address, to_address, amount"
            exit 1
        fi
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            transfer \
            --from "${ARGS[0]}" \
            --to "${ARGS[1]}" \
            --amount "${ARGS[2]}"
        ;;
    "balance")
        if [ ${#ARGS[@]} -ne 1 ]; then
            echo "Error: balance requires 1 argument: account_address"
            exit 1
        fi
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            balance \
            --account "${ARGS[0]}"
        ;;
    "total_supply")
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            total_supply
        ;;
    "total_staked")
        soroban contract invoke \
            --id "$CONTRACT_ID" \
            --source "$SOURCE_ACCOUNT_SECRET" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            -- \
            total_staked
        ;;
    *)
        echo "Error: Unknown method '$METHOD'"
        echo "Available methods:"
        echo "  initialize"
        echo "  issue_certificate"
        echo "  revoke_certificate"
        echo "  get_certificate"
        echo "  is_certified"
        echo "  verify"
        echo "  batch_verify"
        echo "  stake"
        echo "  unstake"
        echo "  slash"
        echo "  get_stake"
        echo "  transfer"
        echo "  balance"
        echo "  total_supply"
        echo "  total_staked"
        exit 1
        ;;
esac

echo "Invocation complete!"
