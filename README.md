[comment]: # Webapp adapted from: https://internetcomputer.org/docs/current/developer-docs/integrations/internet-identity/integrate-identity/

## Usage

Building and deploying the canisters:
```
npm install
dfx start --clean --background #start the local replica
dfx canister create --all
mkdir src/civic_canister_backend/dist
dfx build
dfx canister install --all
dfx deploy 
```

This deploys the ```civic_canister_backend```, ```internet_identity``` and the frontend canister to the local replica.

To run the tests:
```
cargo test --test integration_tests
```

