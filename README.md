[comment]: # Webapp adapted from: https://internetcomputer.org/docs/current/developer-docs/integrations/internet-identity/integrate-identity/

## Prerequisites
Make sure to install dfx: 
```bash
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
```
Add Rust target and ic-wasm:
```
rustup target add wasm32-unknown-unknown
cargo install ic-wasm
```



## Usage
### Setup
Initialize/Update the submodule with 
```
git submodule init
git submodule update 
```
(alternatively clone with `--recurse-submodules`)

Start the local ICP replica (if needed) 
```
dfx start --clean --background 
```

Create the canisters 
```
dfx canister create --all
```

### Configure the correct Alternative Frontends
Get the ID of the Civic Backend canister
```
dfx canister id civic_canister_backend
```

Put this as the `canisterId` inside the `src/civic_canister_frontend/index.ts` AND `src/relying_canister_frontend/src/index.ts`:
```
const canisterId = "canister-id-here" 
```
This sets up the canister login with the correct `derivationOrigin` that the vc-flow call inside `src/relying_canister_frontend/src/index.ts` will later be pointed to

(NOTE: You should be able to get these IDs from environmental variables as well, like ```const local_ii_url = `http://${process.env.INTERNET_IDENTITY_CANISTER_ID}.localhost:4943`;``` but that's not working for the `CIVIC_CANISTER_BACKEND_ID`)

Get the ID of the civic frontend canister 
```
dfx canister id civic_canister_frontend
```
write it into the `src/civic_canister_backend/dist/.well-known/ii-alternative-origins` file:
```
{
    "alternativeOrigins": ["http://${ID-here}.localhost:4943"]
}
```

### Deploying 
Deploy II
```
dfx deploy internet_identity
```

Now build & deploy the Civic Frontend:
```
cd src/civic_canister_frontend
yarn install
yarn build
dfx deploy civic_canister_frontend
cd ../..
```

Build & Deploy Civic Backend Canister (using the local `ic_rootkey`):
```
src/civic_canister_backend/deploy-civic.sh
```


RP Frontend: 
```
cd src/relying_canister_frontend
yarn install
yarn build
./deploy-rp.sh
```

### Tests
To run the tests:
```
cargo test --test integration_tests
```

## Demo Flow

1. Open the ```civic_canister_frontend``` using the second URL (looks like so: `http://${canister-id}.localhost:4943/`). Login & issue the credential. The credential is now stored against the principal that's printed. 
2. Open the ```relying_canister_frontend``` using the second URL again. Login and request the VC through Internet Identity. 

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


