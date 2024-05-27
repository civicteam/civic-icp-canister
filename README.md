# CI Build and Test (Local setup)

This project provides steps to set up and run the project locally. Follow the steps below to configure your environment, build the project, and run tests.

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
```

```markdown
# Civic Canister Deployment

This project involves deploying the Civic frontend, relying and backend canisters on the Internet Computer (IC) network.

## Prerequisites

- Ensure you have an ICP wallet to add cycles.
- Create a new identity for secure mainnet operations.

## Steps to Deploy

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

### 3. Deploy the Frontend Canister

#### Create the Frontend Canister

```bash
dfx canister --network ic create civic_canister_frontend
```

#### Top-Up the Frontend Canister with Cycles

```bash
dfx canister --network ic deposit-cycles 1000000000000 civic_canister_frontend
```

Adjust the number of cycles as needed.

#### Deploy the Frontend Canister

```bash
dfx deploy civic_canister_frontend --network ic --identity mainnet_identity
```

### 3. Relying canister

Follow the sames steps that you followed when deploying the frontend canister

### 4. Deploy the Backend Canister

#### Create the Backend Canister

```bash
dfx canister --network ic create civic_canister_backend
```

#### Top-Up the Backend Canister with Cycles

```bash
dfx canister --network ic deposit-cycles 1000000000000 civic_canister_backend
```

Adjust the number of cycles as needed.

#### Deploy the Backend Canister

Run the deploy-civic.sh script

## Summary

1. **Create and use a secure identity:**

2. **Ensure the wallet has enough cycles:**

3. **Deploy the Frontend Canister:**
4. **Deploy the Relying canister Canister:**

5. **Deploy the Backend Canister:**
