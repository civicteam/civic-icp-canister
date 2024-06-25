#!/usr/bin/env bash

set -euo pipefail

# source .env.local 

#########
# USAGE #
#########

function title() {
    echo "Provisioning Civic canister" >&2
}

function usage() {
    cat >&2 << EOF

Usage:
  $0 [--ii-canister-id CANISTER_ID] [--dfx-network NETWORK]

Options:
  --ii-canister-id CANISTER_ID  The canister ID to use as IDP, defaults to the local internet_identity canister
  --dfx-network NETWORK         The network to use (typically "local" or "ic"), defaults to "local"
  --issuer-canister CANISTER    The canister to configure (name or canister ID), defaults to "issuer"
EOF
}

function help() {
    cat >&2 << EOF

The issuer canister needs some information to operate correctly. This reads data
from the replica to ensure the issuer is provisioned correctly.
EOF
}

II_CANISTER_ID=
ADMIN_PRINCIPAL_ID=tglqb-kbqlj-to66e-3w5sg-kkz32-c6ffi-nsnta-vj2gf-vdcc5-5rzjk-jae

while [[ $# -gt 0  ]]
do
    case "$1" in
        -h|--help)
            title
            usage
            help
            exit 0
            ;;
        --ii-canister-id)
            II_CANISTER_ID="${2:?missing value for '--ii-canister-id'}"
            shift; # shift past --ii-canister-id & value
            shift;
            ;;
        --dfx-network)
            DFX_NETWORK="${2:?missing value for '--dfx-network'}"
            shift; # shift past --dfx-network & value
            shift;
            ;;
        --issuer-canister)
            ISSUER_CANISTER_ID="${2:?missing value for '--issuer-canister-id'}"
            shift; # shift past --issuer-canister & value
            shift;
            ;;
        *)
            echo "ERROR: unknown argument $1"
            usage
            echo
            echo "Use '$0 --help' for more information"
            exit 1
            ;;
    esac
done

# Make sure we always run from the repo's root
SCRIPTS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPTS_DIR/.."

DFX_NETWORK="${DFX_NETWORK:-local}"
II_CANISTER_ID="${II_CANISTER_ID:-$(dfx canister id internet_identity --network "$DFX_NETWORK")}"
ISSUER_CANISTER_ID="${ISSUER_CANISTER_ID:-$(dfx canister id civic_canister_backend --network "$DFX_NETWORK")}"
CIVIC_FRONTEND_CANISTER_ID="${CIVIC_FRONTEND_CANISTER_ID:-$(dfx canister id civic_canister_frontend --network "$DFX_NETWORK")}"
if [ "$DFX_NETWORK" = "local" ]; then
  REPLICA_SERVER_PORT=$(dfx info webserver-port)
  ISSUER_DERIVATION_ORIGIN="http://${ISSUER_CANISTER_ID}.localhost:${REPLICA_SERVER_PORT}"
  ISSUER_FRONTEND_HOSTNAME="http://${ISSUER_CANISTER_ID}.localhost:${REPLICA_SERVER_PORT}"
fi
if [ "$DFX_NETWORK" = "ic" ]; then
  ISSUER_DERIVATION_ORIGIN="https://${ISSUER_CANISTER_ID}.icp0.io"
  ISSUER_FRONTEND_HOSTNAME="https://${ISSUER_CANISTER_ID}.icp0.io"
fi


echo "Using DFX network: $DFX_NETWORK" >&2
echo "Using II canister: $II_CANISTER_ID" >&2
echo "Using issuer canister: $ISSUER_CANISTER_ID" >&2
echo "Using derivation origin: $ISSUER_DERIVATION_ORIGIN" >&2
echo "Using frontend hostname: $ISSUER_FRONTEND_HOSTNAME" >&2

# At the time of writing dfx outputs incorrect JSON with dfx ping (commas between object
# entries are missing).
# In order to read the root key we grab the array from the '"root_key": [...]' bit, the brackets
# to match what candid expects ({}), replace the commas between array entries to match
# what candid expects (semicolon) and annotate the numbers with their type (otherwise dfx assumes 'nat'
# instead of 'nat8').
rootkey_did=$(dfx ping "$DFX_NETWORK" \
    | sed -n 's/.*"root_key": \[\(.*\)\].*/{\1}/p' \
    | sed 's/\([0-9][0-9]*\)/\1:nat8/g' \
    | sed 's/,/;/g')

echo "Parsed rootkey: ${rootkey_did:0:20}..." >&2

# Add also dev server to alternative origins when deploying locally
if [ "$DFX_NETWORK" = "local" ]; then
  ALTERNATIVE_ORIGINS="\"http://$CIVIC_FRONTEND_CANISTER_ID.localhost:4943\", \"https://icp-civic-sign-frontend-demo-civic.vercel.app/\""
  else
  ALTERNATIVE_ORIGINS="\"https://$CIVIC_FRONTEND_CANISTER_ID.icp0.io\", \"https://icp-civic-sign-frontend-demo-civic.vercel.app/\""
fi

echo "Using Alternative Origin: $ALTERNATIVE_ORIGINS $ISSUER_FRONTEND_HOSTNAME"

# Adjust issuer's .well-known/ii-alternative-origins to contain FE-hostname of local/dev deployments.
# We had a problem with `sed` command in CI. This is a hack to make it work locally and in CI.
mv src/civic_canister_backend/dist/.well-known/ii-alternative-origins ./ii-alternative-origins-template
cat ./ii-alternative-origins-template | sed "s+ISSUER_FE_HOSTNAME_PLACEHOLDER+$ALTERNATIVE_ORIGINS+g"  > src/civic_canister_backend/dist/.well-known/ii-alternative-origins
rm ./ii-alternative-origins-template

dfx deploy civic_canister_backend --network "$DFX_NETWORK" --argument '(
    opt record {
        idp_canister_ids = vec { principal "'"$II_CANISTER_ID"'" };
        ic_root_key_der = vec '"$rootkey_did"';
        derivation_origin = "'"$ISSUER_DERIVATION_ORIGIN"'";
        frontend_hostname = "'"$ISSUER_FRONTEND_HOSTNAME"'";
        admin = principal "'"$ADMIN_PRINCIPAL_ID"'";
        authorized_issuers = vec { principal "'"$ADMIN_PRINCIPAL_ID"'" };
    }
)'
# Revert changes
git checkout src/civic_canister_backend/dist/.well-known/ii-alternative-origins