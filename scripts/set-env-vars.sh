#!/usr/bin/env bash

#set the env variables for the frontend canisters
export VITE_LOCAL_CIVIC_FRONTEND_CANISTER_ID=$(dfx canister id civic_canister_frontend)
export VITE_LOCAL_INTERNET_IDENTITY_CANISTER_ID=$(dfx canister id internet_identity)
export VITE_LOCAL_CIVIC_BACKEND_CANISTER_ID=$(dfx canister id civic_canister_backend)
export VITE_LOCAL_RELYING_FRONTEND_CANISTER_ID=$(dfx canister id relying_canister_frontend)
