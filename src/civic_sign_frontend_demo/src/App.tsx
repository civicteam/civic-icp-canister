import React, { useEffect, useState, useCallback, useMemo } from 'react';
import { Principal } from '@dfinity/principal';
import { PrincipalService } from './service/PrincipalService.js';
import { config } from './config.js';
import ICPCredentialCheckButton, { CredentialCheckResponse } from '@civic/icp-gateway-react-ui';

function App() {
  const [principal, setPrincipal] = useState<Principal | undefined>(undefined);
  const { gatekeeperNetwork } = config;

  const [urlCode, setUrlCode] = useState<string | null>(null);

  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get('code');
    setUrlCode(code);
  }, [window.location.search]);

  useEffect(() => {
    console.log('Config:', config);
  }, [config]);

  const handleLogin = useCallback(async () => {
    const principalService = new PrincipalService({
      identityProvider: config.internetIdentityUrl,
    });

    try {
      const userPrincipal = await principalService.requestPrincipal();
      if (userPrincipal) {
        setPrincipal(userPrincipal);
      }
    } catch (error) {
      console.error('Error logging in:', error);
    }
  }, []);

  const handleCredentialCheck = useCallback(async (credential?: CredentialCheckResponse, error?: Error) => {
    console.log('handleCredentialCheck', credential, error);
  }, []);

  return (
    <main>
        <img src="/logo2.svg" alt="DFINITY logo" />
        {principal && <h1>Welcome to the ICP Relying Canister</h1>}
        {principal && <p>Logged in as {principal?.toText()}</p>}
        {principal
          ? <ICPCredentialCheckButton principal={principal} gatekeeperNetwork={gatekeeperNetwork} onCredentialCheck={handleCredentialCheck} /> 
          : urlCode ? <></> : <button onClick={handleLogin}>Login</button>}
        {urlCode && <img src={'https://www.icegif.com/wp-content/uploads/2023/01/icegif-162.gif'} alt="Cool GIF for active status" style={{ width: '100px', height: '100px' }} />}
    </main>
  );
}

export default App;
