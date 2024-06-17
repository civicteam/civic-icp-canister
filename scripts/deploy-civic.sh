#!/bin/bash

# Function to check if canister IDs exist
check_canister_id() {
  local canister_name=$1
  local network=$2
  dfx canister id $canister_name --network $network >/dev/null 2>&1
  if [ $? -ne 0 ]; then
    return 1
  else
    return 0
  fi
}

# Function to deploy a canister
deploy_canister() {
  local canister_name=$1
  local network=$2
  local identity=$3
  local log_file="deploy.log"

  # Deploy the canister
  echo "Deploying $canister_name on network $network..."
  if [ -n "$identity" ]; then
    dfx deploy $canister_name --network $network --identity $identity >>$log_file 2>&1
  else
    dfx deploy $canister_name --network $network >>$log_file 2>&1
  fi
}

# Main deployment script
main() {
  local network=${1:-local}  # Default network to 'local' if not provided
  local identity=""          # Default identity to empty
  local log_file="deploy.log"

  # Clear previous log file
  >$log_file

  # If deploying on mainnet, set identity
  if [ "$network" = "ic" ]; then
    identity="mainnet_identity"  # Set identity for mainnet operations
  fi

  # Start the local DFX environment if deploying locally
  if [ "$network" = "local" ]; then
    echo "Starting local DFX environment..."
    dfx start --clean --background >>$log_file 2>&1
  fi

  # Check if canister IDs exist, create if not
  if ! check_canister_id "civic_canister_backend" $network; then
    echo "Creating canister IDs on network $network..."
    dfx canister --network $network create --all >>$log_file 2>&1
  fi

  # Build the canisters to ensure all necessary files are generated
  echo "Building canisters..."
  dfx build >>$log_file 2>&1

  # Export environment variables
  if [ "$network" = "ic" ]; then
    echo "Setting environment variables for mainnet..."
    . scripts/set-env-vars-production.sh >>$log_file 2>&1
  else
    echo "Setting environment variables for local deployment..."
    . scripts/set-env-vars.sh >>$log_file 2>&1
  fi

  # Deploy internet_identity canister if deploying locally
  if [ "$network" = "local" ]; then
    echo "Deploying internet_identity canister for local environment..."
    dfx deploy internet_identity --network $network >>$log_file 2>&1
  fi

  # Deploy frontend canister
  deploy_canister "civic_canister_frontend" $network $identity

  # Deploy relying canister
  deploy_canister "relying_canister_frontend" $network $identity

  # Deploy backend canister
  echo "Deploying civic_canister_backend on network $network..."
  if [ "$network" = "ic" ]; then
    DFX_NETWORK=ic ./scripts/deploy-civic-backend.sh >>$log_file 2>&1  # Use specific script for mainnet
  else
    dfx deploy civic_canister_backend --network $network >>$log_file 2>&1  # Deploy for local or other networks
  fi

  echo "Deployment completed successfully."
  echo "Please check deploy.log for details."
}

# Execute main function with provided network argument
main $1
