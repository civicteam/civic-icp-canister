#!/usr/bin/env bash
export VITE_ENV=production
#set the env variables for the frontend canisters
export VITE_CIVIC_FRONTEND_CANISTER_ID=$(dfx canister id civic_canister_frontend --network ic)
export VITE_INTERNET_IDENTITY_CANISTER_ID=$(dfx canister id internet_identity --network ic)
export VITE_CIVIC_BACKEND_CANISTER_ID=$(dfx canister id civic_canister_backend --network ic)
export VITE_RELYING_FRONTEND_CANISTER_ID=$(dfx canister id relying_canister_frontend --network ic)
export VITE_HOST=localhost:4943

# Verify the variables are set
echo "VITE_CIVIC_FRONTEND_CANISTER_ID=$VITE_CIVIC_FRONTEND_CANISTER_ID"
echo "VITE_INTERNET_IDENTITY_CANISTER_ID=$VITE_INTERNET_IDENTITY_CANISTER_ID"
echo "VITE_CIVIC_BACKEND_CANISTER_ID=$VITE_CIVIC_BACKEND_CANISTER_ID"
echo "VITE_RELYING_FRONTEND_CANISTER_ID=$VITE_RELYING_FRONTEND_CANISTER_ID"
echo "VITE_HOST=$VITE_HOST"
echo "VITE_ENV=$VITE_ENV"
