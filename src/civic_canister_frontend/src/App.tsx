import React, { useEffect, useState, useCallback } from 'react';
import { Principal } from '@dfinity/principal';
import { CredentialService, Credential } from './service/CredentialService';
import { PrincipalService } from './service/PrincipalService';
import { config } from './config';

function App() {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [principal, setPrincipal] = useState<Principal | undefined>(undefined);
  const [credentialService, setCredentialService] = useState<CredentialService>();

  useEffect(() => {
    console.log('Config:', config);
  }, [config]);

  useEffect(() => {
    const { civicBackendCanisterId, dummyCivicSampleKey } = config;
    const service = new CredentialService({
      civicBackendCanisterId,
      dummyCivicSampleKey,
    });
    setCredentialService(service);
  }, []);

  const handleLogin = useCallback(async () => {
    const principalService = new PrincipalService({
      identityProvider: config.internetIdentityUrl,
      derivationOrigin: config.civicBackendCanisterUrl,
    });

    try {
      const userPrincipal = await principalService.requestPrincipal();
      if (userPrincipal) {
        setIsLoggedIn(true);
        setPrincipal(userPrincipal);
      }
    } catch (error) {
      console.error('Error logging in:', error);
    }
  }, []);

  const storeCredential = useCallback(async () => {
    if (principal && credentialService) {
      try {
        const result = await credentialService.addCredential(principal, credential);
        console.log('Credential stored successfully:', result);
      } catch (error) {
        console.error('Error storing credential:', error);
      }
    } else {
      console.error('Credential service or principal not available');
    }
  }, [principal, credentialService]);

  return (
    <main>
      <img src="/logo2.svg" alt="DFINITY logo" />
      {isLoggedIn && <h1>Welcome to the ICP Civic Canister</h1>}
      {isLoggedIn && <p>Logged in as {principal?.toText()}</p>}
      {isLoggedIn && <button onClick={storeCredential}>Store Credential</button>}
      {!isLoggedIn && <button onClick={handleLogin}>Login</button>}
    </main>
  );
}

const mixedClaim = {
  claims: [
    ["passType", { Text: "tigoYhp9SpCDoCQmXGj2im5xa3mnjR1zuXrpCJ5ZRmi" }],
    ["status", { Text: "active" }],
    ["expirationDate", { Number: 1000 + 365 * 24 * 60 * 60 * 1000 }]
  ]
};


const credential = {
  id: "urn:uuid:6a9c92a9-2530-4e2b-9776-530467e9bbe0",
  type_: ["VerifiableCredential", "CivicPass"],
  context: ["https://www.w3.org/2018/credentials/v1", "https://www.w3.org/2018/credentials/examples/v1"],
  claim: [mixedClaim]
};

export default App;