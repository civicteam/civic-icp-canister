[comment]: # Webapp adapted from: https://internetcomputer.org/docs/current/developer-docs/integrations/internet-identity/integrate-identity/

## Usage
Make sure to install dfx: 
```bash
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
```
Add Rust target and ic-wasm:
```
rustup target add wasm32-unknown-unknown
cargo install ic-wasm
```


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

## ICP Notes

### Flow for the user sign-in 
*(sk = secret key, pk = public key)*

As their "Internet Identity", a user sees a number like 193431. This is called a identity anchor. 

To register a new Internet Identity (identity anchor):
- The user's security device generates a keypair that acts as the master keypair for this identity anchor
- The secure device signs a delegation from from the master public key `master_pk` to the session public key `session_pk` such that the latter is valid wrt the `master_pk` (see delegation chain below)

Now the browser can use this session keypair to sign messages using a delegated signature under the principal derived from the `master_pk` as `SHA224(master_pk) · 0x02`. *All subsequent operations for this identity anchor must be authenticated under one of the principals that are associated like this with the anchor.*

To sign into the Civic Canister: 
1. Get a session keypair with valid delegation:
    - If the browser memory contains none, it will generate a new one as described above. 
2. Use the session keypair to sign into the dApp:
    - Sign an ingress message under the principal described above
    - Sign a second ingress message containing the relying canister ID and session public key `session_pk` 
    - The II canister computes a unique self-authenticating principal for this canister dApp. It uses a secret salt contained in its storage in order to prevent tracking of users by linking their different app identities together. 
    - The II canister returns the "canister signature" that certifies that the `session_pk` may be used with the relying canister ID
    
After this process, the user is logged into the app using a temporary session and without their Internet Identity being able to be linked to other apps. 

Source: https://wiki.internetcomputer.org/wiki/Internet_Identity

### Canister Call Authentication Fields
*(dk = delegated public key, wrt = with regards to)*

```sender``` - Principal which identifies user who issued the request
```sender_pubkey``` - public key used to authenticate the request (in the future a user may have more than one pk)
```sender_sig``` - signature of request
```sender_delegation``` - chain of delegation, example
 *dk1 signed wrt pk1, where pk1 = sender_pubkey -> dk2 signed wrt dk1 -> dk3 signed wrt dk2, where sender_sig is valid wrt to dk3*
 
 The ```sender_pubkey``` must authenticate the sender principal, ie ```sender_sig``` verifies the authenticity of the call against the public key (*self-authenticating principal*)

Source:
https://internetcomputer.org/docs/current/references/ic-interface-spec#authentication
https://internetcomputer.org/docs/current/developer-docs/web-apps/independently-verifying-ic-signatures

## Sequence Diagrams 
![canister drawio](https://github.com/civicteam/icp-civic-canister/assets/66886792/72ef5395-5751-4597-b25c-878b50ef8a85)


