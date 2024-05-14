import React, { useEffect, useState, useCallback } from 'react';
import { Principal } from '@dfinity/principal';
import { CredentialService } from './service/CredentialService.js';
import { PrincipalService } from './service/PrincipalService.js';
import { config } from './config.js';

function App() {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [principal, setPrincipal] = useState<Principal | undefined>(undefined);
  const [credentialService, setCredentialService] = useState<CredentialService>();

  useEffect(() => {
    const service = new CredentialService(config);
    setCredentialService(service);
  }, []);

  const handleLogin = useCallback(async () => {
    const principalService = new PrincipalService({
      identityProvider: config.internetIdentityUrl,
      derivationOrigin: config.relyingFrontendCanisterUrl,
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

  const retrieveCredential = useCallback(async () => {
    if (principal && credentialService) {
      try {
        const result = await credentialService.getCredentials(principal);
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
      {isLoggedIn && <h1>Welcome to the ICP Relying Canister</h1>}
      {isLoggedIn && <p>Logged in as {principal?.toText()}</p>}
      {isLoggedIn && <button onClick={retrieveCredential}>Retrieve Credential</button>}
      {!isLoggedIn && <button onClick={handleLogin}>Login</button>}
    </main>
  );
}

export default App;
