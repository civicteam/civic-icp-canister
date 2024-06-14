import React, { useEffect, useState, useCallback } from 'react';
import { Principal } from '@dfinity/principal';
import { CredentialConfig, CredentialService } from './service/CredentialService.js';
import { PrincipalService } from './service/PrincipalService.js';
import { config } from './config.js';
import { Chain, CivicSignProveFactory, SignedProof } from '@civic/civic-sign';
import axios, { AxiosError, AxiosResponse } from 'axios';
import { pollUntilConditionMet } from './retries.js';
import { FaCheckCircle } from 'react-icons/fa';
import "./index.scss";


function App() {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [principal, setPrincipal] = useState<Principal | undefined>(undefined);
  const [credentialService, setCredentialService] = useState<CredentialService>();
  const [signSuccess, setSignSuccess] = useState<string | undefined>(undefined);
  const [authSuccess, setAuthSuccess] = useState(false);
  const [authToken, setAuthToken] = useState<string | undefined>(undefined);

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


  const onAuth = async (principal: string, config: {
    internetIdentityUrl: string,
    internetIdentityCanisterId: string,
  }) => {
    // get challenge nonce
    const nonce = await getNonce('dev');
    console.log(nonce);

    // sign challenge nonce
    const civicSignProve = CivicSignProveFactory.createWithICPWallet(
      { principal }, { url: config.internetIdentityUrl });
    const proof = await civicSignProve.requestProof(nonce.nonce);
    console.log(proof);
    setSignSuccess(proof.proof);

    // Simulate token retrieval
    const token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

    // // send to civic-sign-backend
    // const token = await getCivicSignAuthToken({
    //   did: `did:icp:v0:${principal}`,
    //   address: principal,
    //   chain: Chain.ICP.toString(),
    //   network: '',
    //   proof,
    //   nonceTimestamp: nonce.timestamp
    // })


    setAuthSuccess(true);
    // if (token) {
    setAuthSuccess(true);
    setAuthToken(token);
    // }
  };

  return (
    <main>
      <img src="/logo2.svg" alt="DFINITY logo" />
      {isLoggedIn && <h1>Welcome to Civic Pass</h1>}
      {isLoggedIn && <p>Logged in as {principal?.toText()}</p>}
      {!isLoggedIn && <button onClick={handleLogin}>Login</button>}
      {isLoggedIn && !signSuccess && <button onClick={() => onAuth(principal?.toString() as string, config)}>Auth</button>}
      {signSuccess && (
        <div>
          <p>Created POWO: {signSuccess}</p>
        </div>
      )}
      {authSuccess && (
        <div className="auth-success-box" style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
          <div style={{ display: 'flex', flexDirection: 'row', alignItems: 'center' }}>
            <FaCheckCircle color="green" className="checkmark-icon" />
            <span style={{ color: 'green', marginRight: '10px' }}>Verified POWO</span>
          </div>
          <p className="auth-token">Civic Sign Authentication Token: {authToken}</p>
        </div>
      )}    </main>
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


type Nonce = { nonce: string; timestamp: number };
const getNonce = async (civicPassApiStage: string): Promise<Nonce> => {
  const response = await axios.get<Nonce>(`https://dev.api.civic.com/sign-${civicPassApiStage}/nonce`);
  return response.data;
};

export const getCivicSignAuthToken = async (
  body: {
    did: string;
    address: string;
    chain: string;
    network: string;
    proof: SignedProof;
    nonceTimestamp: number
  },
  civicSignBackendStage = 'dev'
): Promise<string> => {
  // the authenticate stage sometimes returns a 503, so use retries
  const response = await pollUntilConditionMet(
    async () => {
      try {
        return await axios.post<{ token: string }>(
          //`https://dev.api.civic.com/sign-${civicSignBackendStage}/authenticate`,
          'http://localhost:3000/dev/authenticate',
          body
        );
      } catch (error) {
        const axiosError = error as AxiosError;
        return { status: axiosError?.response?.status } as AxiosResponse;
      }
    },
    (response: AxiosResponse) => response.status < 500
  );
  return response.data.token;
};
