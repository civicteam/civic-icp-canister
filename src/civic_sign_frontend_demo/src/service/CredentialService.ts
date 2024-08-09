// src/service/CredentialService.ts

import { Actor, HttpAgent } from "@dfinity/agent";
import { idlFactory as civic } from "../declarations/civic_canister_backend/civic_canister_backend.did.js";
import { Principal } from "@dfinity/principal";
import { requestVerifiablePresentation } from "@dfinity/verifiable-credentials/request-verifiable-presentation";

export type CredentialConfig = {
  civicBackendCanisterUrl: string;
  dummyCivicSampleKey: Uint8Array;
  internetIdentityUrl: string;
  civicBackendCanisterId: string;
}

export class CredentialService {
  private _agent?: HttpAgent;

  constructor(private config: CredentialConfig) {}

  async getCredentialsFromCanister(principal: string): Promise<any | Error> {
    const agent = new HttpAgent({ host: 'https://ic0.app' });
    agent.fetchRootKey();
    const actor = Actor.createActor(civic, {
      agent,
      canisterId: '73ncn-4qaaa-aaaag-alddq-cai',
    });
    try {
      const vc = await actor.get_all_credentials(Principal.fromText(principal));
      console.log('VC:', vc);
      return vc;
    } catch (error) {
      console.log('Error:', error);
      return Error(error as string);
    }
  };

  // Retrieve all credentials for a given principal
  async getCredentials(principal: Principal): Promise<void> {
    try {
      const issuerData = {
        origin: this.config.civicBackendCanisterUrl,
        canisterId: Principal.fromText(this.config.civicBackendCanisterId),
      };

      const credentialData = {
        credentialSpec: {
          credentialType: 'CivicPass',
          arguments: {}
        },
        credentialSubject: principal
      };

      console.log('Requesting Verifiable Credentials...', await this.getCredentialsFromCanister(principal.toText()));

      const onSuccess = (response: any) => 
        console.log('VC Request Successful:', response);
      
      const onError = (error: any) =>
        console.error('VC Request Failed:', error);
      
      const identityProvider =  new URL(this.config.internetIdentityUrl);
      
      const derivationOrigin = undefined;

      console.log('Requesting Verifiable Presentation...', this.config);
      
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
