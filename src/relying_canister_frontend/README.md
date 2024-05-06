# Relying Canister Verified Credential Consumption Example 

1. User logs into the relying party canister using Internet Identity, get their principal from the point of view from the relying party 
2. Get the credential of this user: Open the Internet Identity vc-flow window and post the Credential Request using the `@dfinity/verifiable-credentials` library 
3. Print the credential 

## Usage

```bash
$ yarn install 
$ yarn dev 
```

