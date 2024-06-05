# Introduction

The Civic Canister is a component of a decentralized application deployed on the Internet Computer (IC) network by Civic. It is designed to handle secure storage and retrieval of credentials under Internet Identity, the authentication service for the Internet Computer. Internet Identity provides different, unlinkable identities for each app a user logs in to. 

Civic (or other issuers) can issue credentials for the user as part of its identity verification flow. 3rd parties integrating Civic Pass on ICP can request these credentials from the Civic Canister. Through Internet Identity, the credential can be shared securely between the two dApps without linking the user's identities between the two dApps. 

![Overview of how the Civic Canister works](<Civic Canister Flow.png>)

As you can see in the diagram, the user has two "versions" of their Internet Identity, Identity A and Identity B. Identity A is the one used in the Civic Canister while Identity B is the one used by the 3rd party dApp (Relying Party). When the credential data is shared between the two dApps, the two identities are not linked to each other, thanks to the attribute sharing of Internet Identity which the Civic Canister implements. 

This repository contains the code to demo the above flow. `civic_canister_backend` contains the code for the Civic Canister, `relying_canister_frontend` contains the code for the example 3rd party, and `civic_canister_frontend` contains the code for an example frontend through which credentials can be issued to the canister. See [CI Build and Test (Local Setup)](#ci-build-and-test-local-setup) for instructions how to run the demo locally. 


## What does the Credential issued by the Civic Canister looks like? 
The credentials issued by the Civic Canister adhere to the W3C Verifiable Credentials specification, ensuring compatibility and interoperability with various systems and platforms. 

Here is an example of a credential the Civic Canister can issue:

```
{
  "@context": [
    "https://www.w3.org/ns/credentials/v2",
    "https://www.w3.org/ns/credentials/examples/v2"
  ],
  "id": "urn:uuid:6a9c92a9-2530-4e2b-9776-530467e9bbe0",
  "type": ["VerifiableCredential", "CivicUniquenessPass"],
  "issuer": "did:icp:v0:tglqb-kbqlj-to66e-3w5sg-kkz32-c6ffi-nsnta-vj2gf-vdcc5-5rzjk-jae",
  "expiry": "2024-04-04T00:00:00Z",
  "credentialSubject": {
    "id": "did:icp:user-principal",
    "CivicPass": {
      "id": "did:example:c276e12ec21ebfeb1f712ebc6f1",
      "name": "Civic Uniqueness Pass",
      "expiry": "2024-04-04T00:00:00Z"
    }
  }
}
```



# CI Build and Test (Local Setup)

This section provides steps to set up and run the project locally. Follow the steps below to configure your environment, build the project, and run tests.

## Prerequisites

- [Node.js](https://nodejs.org/) (version 14.x or later)
- [Rust](https://www.rust-lang.org/tools/install)
- [DFX](https://smartcontracts.org/docs/developers-guide/install-upgrade-remove.html) (Internet Computer SDK)

## Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-repository.git
   cd your-repository
   ```

2. **Install DFX**:
   Follow the official DFX installation guide [here](https://smartcontracts.org/docs/developers-guide/install-upgrade-remove.html).

3. **Install Rust**:
   Follow the official Rust installation guide [here](https://www.rust-lang.org/tools/install).

4. **Install Node.js dependencies**:
   ```bash
   npm install
   ```

5. **Install Rust target and dependencies**:
   ```bash
   rustup target add wasm32-unknown-unknown
   cargo install ic-wasm
   ```

6. **Create canisters and start the local Internet Computer replica**:
   ```bash
   dfx start --clean --background
   dfx canister create --all
   ```

7. **Set environment variables**:
   Make sure the `scripts/set-env-vars.sh` script is executable and run it:
   ```bash
   chmod +x scripts/set-env-vars.sh
   source scripts/set-env-vars.sh
   ```

8. **Deploy the Civic Canister and Internet Identity**:
   ```bash
   ./scripts/deploy-civic.sh
   dfx deploy internet_identity
   ```

9. **Deploy the Civic Frontend Canister**:
    ```bash
    dfx deploy civic_canister_frontend
    ```

10. **Deploy the Relying Party (RP) canister**:
   ```bash
   dfx deploy relying_canister_frontend
   ```

11. **Run tests**:
    Ensure the test state machine binary is executable and run the tests:
    ```bash
    chmod +x ic-test-state-machine
    cargo test --test integration_tests
    ```

## Script for Storing and Fetching Credentials

The following script is located in `./src/civic_frontend_canister`. It demonstrates how to store and fetch credentials from the Civic Canister backend. The script uses environment variables to configure the canister ID and other settings.

### Script Explanation

This script performs the following tasks:

1. **Load Environment Variables**: Uses the `dotenv` package to load environment variables from `.env.local`.
2. **Configure Canister IDs**: Reads the canister IDs from the environment variables.
3. **Dummy Credential Data**: Sets up dummy credential data for testing purposes.
4. **Store Credential**: Stores a credential in the Civic Canister backend.
5. **Fetch Credentials**: Fetches and logs all credentials associated with a specified principal.

### Usage Instructions

1. **Ensure Environment Variables are Set**: Make sure the `.env.local` file contains the necessary environment variables.

2. **Run the Script**:
   Navigate to the directory containing the script and run it using Node.js:
   ```bash
   cd src/civic_canister_frontend
   npm run issue-credential
   ```

## Notes

- Ensure you have the required binaries in the `ic-test-machine-binaries` directory.
- Modify the setup and deployment scripts as needed to suit your project's requirements.

# Civic Canister Mainnet Deployment

This section describes the steps to deploy the Civic frontend, relying and backend canisters on the Internet Computer (IC) network.

## Prerequisites

- Ensure you have an ICP wallet to add cycles.
- Create a new identity for secure mainnet operations.

## Steps to Deploy

### Overview
1. **[Create and use a secure identity](#1-create-and-use-a-secure-identity)**
2. **[Ensure the wallet has enough cycles](#2-ensure-the-wallet-has-enough-cycles)**
3.  **[Create the canister IDs](#3-create-the-canister-ids)**
4. **[Deploy the Frontend Canister](#4-deploy-the-frontend-canister)**
5. **[Deploy the Relying Canister](#5-deploy-the-relying-canister)**

6. **[Deploy the Backend Canister](#6-deploy-the-backend-canister)**

### 1. Create and Use a Secure Identity

Create and use a new identity for secure operations on the mainnet.

### 2. Ensure the Wallet Has Enough Cycles

Check your wallet balance and add cycles if necessary.

#### Check Wallet Balance

```bash
dfx wallet --network ic balance
```

#### Get Wallet Address

```bash
dfx identity get-wallet
```

#### Transfer Cycles to Wallet

Use an external ICP wallet or exchange to transfer cycles to your wallet address. For example, to add 10 trillion cycles:

```bash
# Transfer ICP equivalent to 10 trillion cycles to your wallet address obtained from `dfx identity get-wallet`.
```

### 3. Create the canister IDs 
Verify if the canister IDs were already created by running
```bash
dfx canister id civic_canister_backend --network ic
dfx canister id civic_canister_frontend --network ic
dfx canister id relying_canister_frontend --network ic
```
If this prints the IDs for all three canisters, then they were already created. If not run
```bash
dfx canister --network ic create --all
```

Export the IDs so that the frontend canisters are configured correctly. 
```bash
chmod +x scripts/set-env-vars-production.sh
. scripts/set-env-vars-production.sh
```

### 4. Deploy the Frontend Canister

#### Top-Up the Frontend Canister with Cycles

```bash
dfx canister --network ic deposit-cycles 1000000000000 civic_canister_frontend
```

Adjust the number of cycles as needed.

#### Deploy the Frontend Canister

```bash
dfx deploy civic_canister_frontend --network ic --identity mainnet_identity
```

### 5. Deploy the Relying Canister

Follow the sames steps that you followed when deploying the frontend canister

### 6. Deploy the Backend Canister

#### Top-Up the Backend Canister with Cycles

```bash
dfx canister --network ic deposit-cycles 1000000000000 civic_canister_backend
```

Adjust the number of cycles as needed.

#### Deploy the Backend Canister

To deploy run 
`deploy-civic.sh --dfx-network ic` 

To upgrade run 
`upgrade-civic.sh --dfx-network ic` 


