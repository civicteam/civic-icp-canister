#!/usr/bin/env bash
if [[ "${VITE_ENV}" == "production" ]]; then
  export VITE_CIVIC_FRONTEND_CANISTER_ID=$(dfx canister id civic_canister_frontend --network ic)
  export VITE_INTERNET_IDENTITY_CANISTER_ID=$(dfx canister id internet_identity --network ic)
  export VITE_CIVIC_BACKEND_CANISTER_ID=$(dfx canister id civic_canister_backend --network ic)
  export VITE_RELYING_FRONTEND_CANISTER_ID=$(dfx canister id relying_canister_frontend --network ic)
  export VITE_HOST=icp0.io
else
  export VITE_CIVIC_FRONTEND_CANISTER_ID=$(dfx canister id civic_canister_frontend)
  export VITE_INTERNET_IDENTITY_CANISTER_ID=$(dfx canister id internet_identity)
  export VITE_CIVIC_BACKEND_CANISTER_ID=$(dfx canister id civic_canister_backend)
  export VITE_RELYING_FRONTEND_CANISTER_ID=$(dfx canister id relying_canister_frontend)
  export VITE_HOST=localhost:4943
fi

# Verify the variables are set
echo "VITE_CIVIC_FRONTEND_CANISTER_ID=$VITE_CIVIC_FRONTEND_CANISTER_ID"
echo "VITE_INTERNET_IDENTITY_CANISTER_ID=$VITE_INTERNET_IDENTITY_CANISTER_ID"
echo "VITE_CIVIC_BACKEND_CANISTER_ID=$VITE_CIVIC_BACKEND_CANISTER_ID"
echo "VITE_RELYING_FRONTEND_CANISTER_ID=$VITE_RELYING_FRONTEND_CANISTER_ID"
echo "VITE_HOST=$VITE_HOST"
echo "VITE_ENV=$VITE_ENV"