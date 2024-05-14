// src/service/CredentialService.ts

import { Actor, HttpAgent } from "@dfinity/agent";
import { idlFactory as civic } from "../../../declarations/civic_canister_backend/civic_canister_backend.did.js";
import { Secp256k1KeyIdentity } from "@dfinity/identity-secp256k1";
import type { Principal } from "@dfinity/principal";

export interface Credential {
  id: string;
  type_: string[];
  context: string[];
  issuer: string;
  claim: any; // Array of claims
}

export type CredentialConfig = {
  civicBackendCanisterId: string;
  dummyCivicSampleKey: Uint8Array;
}

export class CredentialService {
  private _agent?: HttpAgent;

  constructor(private config: CredentialConfig) {}

  private get credentialActor() {
    if (!this._agent) {
      const identity = Secp256k1KeyIdentity.fromSecretKey(this.config.dummyCivicSampleKey);
      this._agent = new HttpAgent({ identity });
    }
    this._agent.fetchRootKey();
    return Actor.createActor(civic, {
      agent: this._agent,
      canisterId: this.config.civicBackendCanisterId,
    });
  }

  // Add a credential to the canister
  async addCredential(principal: Principal, credential: Credential): Promise<string | null> {
    try {
      console.log("Adding credential:", credential);
      const result = await this.credentialActor.add_credentials(principal, [credential]);
      console.log("Credential added:", result);
      return result as string;
    } catch (error) {
      console.error("Error adding credential:", error);
      return null;
    }
  }
}
