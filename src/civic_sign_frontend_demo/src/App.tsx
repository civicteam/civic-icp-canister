import React, { useEffect, useState, useCallback } from 'react';
import { Principal } from '@dfinity/principal';
import { CredentialService } from './service/CredentialService.js';
import { PrincipalService } from './service/PrincipalService.js';
import { config } from './config.js';
import { CivicSignProveFactory } from '@civic/civic-sign';
import axios from 'axios';

function App() {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [principal, setPrincipal] = useState<Principal | undefined>(undefined);
  const [credentialService, setCredentialService] = useState<CredentialService>();

  useEffect(() => {
    console.log('Config:', config);
  }, [config]);

  useEffect(() => {
    const service = new CredentialService(config);
    setCredentialService(service);
  }, []);

  const handleLogin = useCallback(async () => {
    const principalService = new PrincipalService({
      identityProvider: config.internetIdentityUrl,
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
      {isLoggedIn && <h1>Welcome to Civic Pass</h1>}
      {isLoggedIn && <p>Logged in as {principal?.toText()}</p>}
      {!isLoggedIn && <button onClick={handleLogin}>Login</button>}
      {isLoggedIn && <button onClick={() => onSignChallenge(principal?.toString() as string)}>Auth</button>}
    </main>
  );
}

export default App;

export const uint8ArrayToHexString = (bytes: Uint8Array | number[]) => {
  if (!(bytes instanceof Uint8Array)) {
    bytes = Uint8Array.from(bytes);
  }
  return bytes.reduce(
    (str, byte) => str + byte.toString(16).padStart(2, "0"),
    ""
  );
};

const onSignChallenge = async (principal: string) => {
  const nonce = await getNonce('dev');
  console.log(nonce);

  const civicSignProve = CivicSignProveFactory.createWithICPWallet(
    { principal });
  const proof = await civicSignProve.requestProof(JSON.stringify(nonce));
  console.log(proof);

  // // SKIP FOR NOW
  // const data = { challenge, delegationIdentity };
  // await fetch("/verify", {
  //   method: "POST",
  //   body: JSON.stringify(data, (_, v) => {
  //     if (typeof v === "bigint") {
  //       // We need to expiration date to be hex string.
  //       return v.toString(16);
  //     }
  //     if (v instanceof Uint8Array) {
  //       // We need the keys to be hex strings.
  //       return uint8ArrayToHexString(v);
  //     }
  //     return v;
  //   }),
  //   headers: new Headers({
  //     "Content-Type": "application/json",
  //   }),
  // });
};

type Nonce = { nonce: string; timestamp: number };
const getNonce = async (civicPassApiStage: string): Promise<Nonce> => {
  const response = await axios.get<Nonce>(`https://dev.api.civic.com/sign-${civicPassApiStage}/nonce`);
  return response.data;
};
