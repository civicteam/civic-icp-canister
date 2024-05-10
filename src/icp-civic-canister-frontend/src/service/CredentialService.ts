// src/service/CredentialService.ts

import { Actor, HttpAgent } from "@dfinity/agent";
import type { Principal } from "@dfinity/principal";
import { idlFactory as civic } from "../../../declarations/icp-civic-canister-backend/icp-civic-canister-backend.did.js"; // Adjust path
import { Secp256k1KeyIdentity } from "@dfinity/identity-secp256k1";

export interface Claim {
  claim_type: string;
  value: string;
}

export interface Credential {
  id: string;
  issuer: string;
  context: string[];
  claims: Claim[];
}

export type CredentialConfig = {
  civicBackendCanisterId: string;
  civicBackendCanisterUrl: string;
}

export class CredentialService {

  get credentialActor() {
    const { civicBackendCanisterId, civicBackendCanisterUrl } = this.config;
    const agent = new HttpAgent();
    const s = agent.fetchRootKey();
    
    return Actor.createActor(civic, {
      agent,
      canisterId: civicBackendCanisterId,
    });
  }

  constructor(readonly config: CredentialConfig) {}

  // Add a credential to the canister
  async addCredential(principal: Principal, credential: Credential): Promise<string | null> {
    try {
      const result = await this.credentialActor.add_credential(principal, credential);
      return result as string;
    } catch (error) {
      console.error("Error adding credential:", error);
      return null;
    }
  }

  // Retrieve all credentials for a given principal
  async getCredentials(principal: Principal): Promise<Credential[] | null> {
    try {
      const credentials = await this.credentialActor.get_credentials(principal);
      return credentials as Credential[];
    } catch (error) {
      console.error("Error getting credentials:", error);
      return null;
    }
  }
}
