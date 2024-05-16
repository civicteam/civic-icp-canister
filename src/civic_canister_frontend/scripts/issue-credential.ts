import { Principal } from "@dfinity/principal";
import * as dotenv from 'dotenv';
import { Secp256k1KeyIdentity } from "@dfinity/identity-secp256k1";
import { Actor, HttpAgent } from "@dfinity/agent";
import { idlFactory as civic } from "../../declarations/civic_canister_backend/civic_canister_backend.did.js";

dotenv.config({ path: '.env.local' });

// Configuration
const civicBackendCanisterId = process.env.VITE_LOCAL_CIVIC_BACKEND_CANISTER_ID ?? "";

console.log('Civic backend canister ID:', civicBackendCanisterId);

const dummyCivicSampleKey = new Uint8Array([
  73, 186, 183, 223, 243, 86, 48, 148, 83, 221, 41, 75, 229, 70, 56, 65, 247,
  179, 125, 33, 172, 58, 152, 14, 160, 114, 17, 22, 118, 0, 41, 243,
]);

// Dummy principal for testing purposes - this is the principal from the vc-flows that represents the user id from the civic POV (the same user that logs into the demo RP)
const principal = Principal.fromText("your-principal-id-here");

// Define the dummy credential
const id = ["id", { Text: "did:example:c276e12ec21ebfeb1f712ebc6f1" }];
const name = ["name", { Text: "Example University" }];
const degreeType = ["degreeType", { Text: "MBA" }];
const alumniOfClaim = {
  claims: [id, name, degreeType]
};
const mixedClaim = {
  claims: [
    ["Is over 18", { Boolean: true }],
    ["name", { Text: "Max Mustermann" }],
    ["alumniOf", { Claim: alumniOfClaim }]
  ]
};

const credential = {
  id: "urn:uuid:6a9c92a9-2530-4e2b-9776-530467e9bbe0",
  type_: ["VerifiableCredential", "VerifiedAdult"],
  context: ["https://www.w3.org/2018/credentials/v1", "https://www.w3.org/2018/credentials/examples/v1"],
  issuer: "https://civic.com",
  claim: [mixedClaim]
};

// Function to store the credential
const storeCredential = async () => {
  const identity = Secp256k1KeyIdentity.fromSecretKey(dummyCivicSampleKey);
  const agent = new HttpAgent({ identity, host: "http://127.0.0.1:4943" });
  await agent.getPrincipal();
  await agent.fetchRootKey();
  const actor = Actor.createActor(civic, {
    agent: agent,
    canisterId: civicBackendCanisterId,
  });

  console.log("Adding credential:", credential);
  const result = await actor.add_credentials(principal, [credential]);
  console.log("Credential added:", result);
  const vc = await actor.get_all_credentials(principal);
  console.log("Credential fetched:", vc);
};

// Run the function
storeCredential();
