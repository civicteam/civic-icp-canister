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

This `README.md` includes instructions for setting up and running the project locally, as well as an explanation and usage instructions for the script in `./src/civic_frontend_canister`. Make sure to replace placeholders like `https://github.com/your-repository.git` and `"principal id here"` with actual values specific to your project.s