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
   git clone https://github.com/civicteam/civic-icp-canister.git
   cd civic-icp-canister
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

## Deploy the canisters 
To simplify the deployment of the canisters we provide a script `deploy-civic.sh` you can find under `scripts`. To execute it make sure that you have run `npm install` beforehand inside the project. 
  ```bash
  ./scripts/deploy-civic.sh local 
  ```

### Manual deployment 
Steps for the manual deployment: 
1. **Create canisters and start the local Internet Computer replica**:
   ```bash
   dfx start --clean --background
   dfx canister create --all
   ```

2. **Set environment variables**:
   Run the `scripts/set-env-vars.sh` script to set the environment variables:
   ```bash
   source scripts/set-env-vars.sh
   ```

3. **Deploy the Civic Canister and Internet Identity**:
   ```bash
   ./scripts/deploy-civic.sh
   dfx deploy internet_identity
   ```


4. **Deploy the Civic Frontend Canister**:

   **Note**: If it fails to deploy, try to deploy the RP canister first and then go back to the Civic Frontend Canister. (If you start with the RP first and it fails, try to deploy the Civic Frontend Canister and then go back to the RP.)

    ```bash
    dfx deploy civic_canister_frontend
    ```


5. **Deploy the Relying Party (RP) canister**:
   ```bash
   dfx deploy relying_canister_frontend
   ```

## Testing
To run the tests ensure the test state machine binary is executable first:
```bash
chmod +x ic-test-state-machine
cargo test --test integration_tests
```

## Storing and Fetching Credentials

The following script is located in `src/civic_frontend_canister`. It demonstrates how to store and fetch credentials from the Civic Canister backend. The script uses environment variables to configure the canister ID and other settings.

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

# Mainnet Deployment

This section describes the steps to deploy the Civic frontend, relying and backend canisters on the Internet Computer (IC) network.

## Prerequisites
- Ensure you can [deploy locally](https://github.com/civicteam/civic-icp-canister/edit/TECH-156__update-script-deployment-fix-paths/README.md#using-the-deploy-script).
- Ensure you have an ICP wallet to add cycles.
- Create a new identity for secure mainnet operations.

## Setup 
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

## Deploy the canisters
You can use the deploy script to deploy the canisters to mainnet in one step. Be sure that you have deployed locally first ([see](https://github.com/civicteam/civic-icp-canister/edit/TECH-156__update-script-deployment-fix-paths/README.md#using-the-deploy-script)), this is a necessity for the deployment to mainnet to work. 
```bash
./scripts/deploy-civic.sh ic 
```

### Manual deployment 
Alternatively you can deploy manually with the following steps: 

3. **[Create the canister IDs](#3-create-the-canister-ids)**

4. **[Deploy the Frontend Canister](#4-deploy-the-frontend-canister)**
5. **[Deploy the Relying Canister](#5-deploy-the-relying-canister)**

6. **[Deploy the Backend Canister](#6-deploy-the-backend-canister)**

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
. scripts/set-env-vars-production.sh
```

### 4. Deploy the Frontend Canister
#### Note: 
If it fails to deploy, try to deploy the RP canister first and then go back to the Civic Frontend Canister. (If you start with the RP first and it fails, try to deploy the Civic Frontend Canister and then go back to the RP.)


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
`DFX_NETWORK=ic deploy-civic.sh` 

To upgrade run 
`DFX_NETWORK=ic upgrade-civic.sh` 


