import React, { useEffect, useState, useCallback, useMemo } from 'react';
import { Principal } from '@dfinity/principal';
import { CredentialService } from './service/CredentialService.js';
import { PrincipalService } from './service/PrincipalService.js';
import { config } from './config.js';
import { GatewayProvider, GatewayStatus, useGateway } from '@civic/icp-gateway-react';
import ICPCredentialCheckButton from '@civic/icp-gateway-react-ui';

function GatewayStatusLookup({ children, isLoggedIn }: { children: React.ReactNode; isLoggedIn: boolean }) {
  const { gatewayStatus } = useGateway();
  const [urlCode, setUrlCode] = useState<string | null>(null);

  useEffect(() => {
    console.log('Gateway status:', gatewayStatus);
    
    // Parse the URL for the 'code' parameter
    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get('code');
    setUrlCode(code);
  }, [gatewayStatus]);

  const retrievePass = () => {
    const currentUrl = window.location.href;
    const encodedRedirectUri = encodeURIComponent(currentUrl);
    const passPortalUrl = `${config.portalUrl}/?redirect_uri=${encodedRedirectUri}`;
    window.location.href = passPortalUrl;
  };

  const isActive = gatewayStatus === GatewayStatus.ACTIVE || urlCode === 'ACTIVE';

  return (
    <>
      {children}
      {isLoggedIn && (
        <div>
          <p>Gateway Status: {!isActive ? 'NOT_FOUND' : 'ACTIVE'}</p>
          {isActive && (
            <div>
              <p>Active status detected!</p>
              <img src={'https://www.icegif.com/wp-content/uploads/2023/01/icegif-162.gif'} alt="Cool GIF for active status" style={{ width: '100px', height: '100px' }} />
            </div>
          )}
        </div>
      )}
      {gatewayStatus === GatewayStatus.NOT_REQUESTED && <button onClick={retrievePass}>Get Pass</button>}
    </>
  );
}

function App() {
  const [principal, setPrincipal] = useState<Principal | undefined>(undefined);
  const icpWallet = useMemo(() => ({ principal: principal?.toText() ?? undefined }), [principal]);
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

  return (
    <main>
        <img src="/logo2.svg" alt="DFINITY logo" />
        {principal && <h1>Welcome to the ICP Relying Canister</h1>}
        {principal && <p>Logged in as {principal?.toText()}</p>}
        {principal 
          ? <ICPCredentialCheckButton principal={principal} gatekeeperNetwork={gatekeeperNetwork} /> 
          : <button onClick={handleLogin}>Login</button>}
    </main>
  );
}

export default App;
