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
```
dfx start --clean --background 
```

Create the canisters 
```
dfx canister create --all
```

Set the environment variables before deploying the backend canister
```
scripts/set-env-vars.sh
```

Deploy the backend canister
```
scripts/deploy-civic.sh
```

Deploy the internet identity canister
```
dfx deploy internet_identity
```

Update the  that are printed in the CLI for the relying and civic dummy canister and then deploy them
```
dfx deploy relying_canister_frontend && dfx deploy relying_canister_frontend
```

### Tests
To run the tests:
```
cargo test --test integration_tests
```

## Demo Flow

1. Open the ```civic_canister_frontend``` using the second URL (looks like so: `http://${canister-id}.localhost:4943/`). Login & issue the credential. The credential is now stored against the principal that's printed. 
2. Open the ```relying_canister_frontend``` using the second URL again. Login and request the VC through Internet Identity. 

## Alternative Frontends 

Setting the correct derivationOrigin/Alternative Frontends of the canisters allows the II backend to correctly convert the principals from the civic POV to the RP POV. The key point is that the the `origin` of the issuer inside the vc-flow call should match the `derivationOrigin` in the login process to the issuer (ie in the civic frontend canister).

1. In the civic frontend, the user logs into the canister using the civic canister backend as `derivationOrigin`. This allows the user's principal to be the same for civic canister backend (since the user is using the _frontend_ canister and they are two separate canisters, II would otherwise use different principals)
2. In the RP, the user wants to request from the issuer of the credentials, namely the civic canister backend. Therefore, the `origin` in the call to the start the vc-flow is set as the civic canister backend. 

3. II must map the generated alias between the RP Canister and the Issuing Canister (in order to provide unlinkability of the user's identities). In the vc-flow, when II is sending a `request_credential` to the backend, the principal that it's using must be the one that the civic canister backend stored the credentials under. By specifying a `derivationOrigin` during the login, II knows to use the same principal as in the login to send to the backend and check for stored credentials. Otherwise there will be an `unauthorized principal` error. 

   
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

