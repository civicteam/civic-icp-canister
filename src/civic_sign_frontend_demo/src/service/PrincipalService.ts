// service/PrincipalService.ts

import { AuthClient } from "@dfinity/auth-client";
import type { Principal } from "@dfinity/principal";

export type PrincipalConfig = {
  identityProvider: string;
}

export class PrincipalService implements PrincipalService {
  authClient: AuthClient | null = null;
  principal: Principal | null = null;

  constructor(public readonly config: PrincipalConfig) {}

  async requestPrincipal(): Promise<Principal | null> {
    if (!this.authClient) {
      this.authClient = await AuthClient.create();
    }

    if (this.principal) {
      return this.principal;
    }

    const loginResult = new Promise((resolve, reject) => {
      const { identityProvider } = this.config;
      this.authClient?.login({
        identityProvider,
        onSuccess: resolve,
        onError: reject
      });
    });

    return loginResult.then(() => {
      this.principal = this.authClient!.getIdentity().getPrincipal();
      return this.principal;
    });
  }
}
