#!/bin/bash

# Function to enable verbose logging if the verbose flag is passed
enable_verbose() {
  if [ "$1" == "verbose" ]; then
    set -x
  fi
}

# Exit immediately if a command exits with a non-zero status
set -e

# Enable verbose output if 'verbose' is passed as the second argument
enable_verbose $2

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
  local log_file="./deploy.log"

  # Deploy the canister
  echo "Deploying $canister_name on network $network..."
  if ! dfx deploy $canister_name --network $network >>$log_file 2>&1; then
    echo "Error: Failed to deploy $canister_name on network $network. Check $log_file for details."
    exit 1
  fi
}

# Function to build canisters with retries
build_canisters_with_retries() {
  local network=$1
  local retries=3
  local count=0
  local success=false
  local log_file="./deploy.log"

  until [ $count -ge $retries ]; do
    echo "Building canisters (attempt $((count+1))/$retries)..."
    if dfx build --network $network >>$log_file 2>&1; then
      success=true
      break
    else
      echo "Error: Failed to build canisters. Retrying..."
      count=$((count+1))
      sleep 2
    fi
  done

  if [ "$success" = false ]; then
    echo "Error: Failed to build canisters after $retries attempts. Check $log_file for details."
    exit 1
  fi
}

# Main deployment script
main() {
  local network=${1:-local}  # Default network to 'local' if not provided
  local log_file="./deploy.log"

  # Clear previous log file
  >$log_file

  # Start the local DFX environment if deploying locally
  if [ "$network" = "local" ]; then
    echo "Starting local DFX environment..."

    # Stop any running dfx instances
    if pgrep -f "dfx start" >/dev/null; then
      echo "Stopping existing DFX instance..."
      dfx stop >>$log_file 2>&1
    fi

    # Start DFX
    if ! dfx start --clean --background >>$log_file 2>&1; then
      echo "Error: Failed to start local DFX environment. Check $log_file for details."
      exit 1
    fi

    # Check if the local DFX environment started successfully
    sleep 5
    if ! pgrep -f "dfx start" >/dev/null; then
      echo "Error: Failed to start local DFX environment. Check $log_file for details."
      exit 1
    fi

    # Deploy internet_identity canister
    echo "Deploying internet_identity canister for local environment..."
    if ! dfx canister create internet_identity --network $network >>$log_file 2>&1; then
      echo "Error: Failed to create internet_identity canister. Check $log_file for details."
      exit 1
    fi
    if ! dfx deploy internet_identity --network $network >>$log_file 2>&1; then
      echo "Error: Failed to deploy internet_identity canister. Check $log_file for details."
      exit 1
    fi
  fi

  # Check if canister IDs exist, create if not
  for canister in civic_canister_backend civic_canister_frontend relying_canister_frontend; do
    if ! check_canister_id $canister $network; then
      echo "Creating canister $canister on network $network..."
      if ! dfx canister --network $network create $canister >>$log_file 2>&1; then
        echo "Error: Failed to create canister $canister. Check $log_file for details."
        exit 1
      fi
    fi
  done

  # Build the canisters with retries to ensure all necessary files are generated
  build_canisters_with_retries $network

  # Export environment variables
  if [ "$network" = "ic" ]; then
    if [ ! -f "./scripts/set-env-vars-production.sh" ]; then
      echo "Error: set-env-vars-production.sh not found."
      exit 1
    fi
    echo "Setting environment variables for mainnet..."
    . ./scripts/set-env-vars-production.sh >>$log_file 2>&1
  else
    if [ ! -f "./scripts/set-env-vars.sh" ]; then
      echo "Error: set-env-vars.sh not found."
      exit 1
    fi
    echo "Setting environment variables for local deployment..."
    . ./scripts/set-env-vars.sh >>$log_file 2>&1
  fi

  # Deploy frontend canister
  deploy_canister "civic_canister_frontend" $network

  # Deploy relying canister
  deploy_canister "relying_canister_frontend" $network

  # Deploy backend canister
  echo "Deploying civic_canister_backend on network $network..."
    if [ ! -f "./scripts/deploy-civic-backend.sh" ]; then
      echo "Error: deploy-civic-backend.sh not found."
      exit 1
    fi
    if ! DFX_NETWORK=$network ./scripts/deploy-civic-backend.sh >>$log_file 2>&1; then
      echo "Error: Failed to deploy civic_canister_backend on network $network. Check $log_file for details."
      exit 1
    fi

  # Stop the local DFX environment if it was started
  if [ "$network" = "local" ]; then
    echo "Stopping local DFX environment..."
    if ! dfx stop >>$log_file 2>&1; then
      echo "Error: Failed to stop local DFX environment. Check $log_file for details."
      exit 1
    fi
  fi

  echo "Deployment completed successfully."
  echo "Please check deploy.log for details."
}

# Execute main function with provided network argument
main $1 $2
