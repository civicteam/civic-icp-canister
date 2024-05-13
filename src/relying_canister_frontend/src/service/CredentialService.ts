// src/service/CredentialService.ts

import { Actor, HttpAgent } from "@dfinity/agent";
import { idlFactory as civic } from "../../../declarations/civic_canister_backend/civic_canister_backend.did.js";
import { Secp256k1KeyIdentity } from "@dfinity/identity-secp256k1";
import type { Principal } from "@dfinity/principal";
import { requestVerifiablePresentation } from "@dfinity/verifiable-credentials/request-verifiable-presentation";

export type CredentialConfig = {
  civicBackendCanisterUrl: string;
  dummyCivicSampleKey: Uint8Array;
  relyingFrontendCanisterUrl: string;
  internetIdentityUrl: string;
}

export class CredentialService {
  private _agent?: HttpAgent;

  constructor(private config: CredentialConfig) {}

  // Retrieve all credentials for a given principal
  async getCredentials(principal: Principal): Promise<void> {
    try {
      const issuerData = {
        "origin": this.config.civicBackendCanisterUrl,
      };

      const credentialData = {
        credentialSpec: {
          credentialType: 'VerifiedAdult',
          arguments: {}
        },
        credentialSubject: principal.toText()
      };

      const onSuccess = (response: any) => 
        console.log('VC Request Successful:', response);
      
      const onError = (error: any) =>
        console.error('VC Request Failed:', error);
      
      const identityProvider = this.config.internetIdentityUrl;
      
      const derivationOrigin = undefined;
      
      const requestParams = {
        onSuccess,
        onError,
        credentialData,
        issuerData,
        identityProvider,
        derivationOrigin
      };
      
      requestVerifiablePresentation(requestParams);
    } catch (error) {
      console.error("Error getting credentials:", error);
    }
  }
}
