import React, { useEffect, useState, useCallback, useMemo } from 'react';
import { Principal } from '@dfinity/principal';
import { CredentialService } from './service/CredentialService.js';
import { PrincipalService } from './service/PrincipalService.js';
import { config } from './config.js';
import { GatewayProvider, GatewayStatus, useGateway } from '@civic/icp-gateway-react';

export const getStatusText = (status: GatewayStatus | null | undefined): string => {
  switch (status) {
    case GatewayStatus.IN_REVIEW:
      return 'Reviewing';
    case GatewayStatus.CHECKING:
      return 'Collecting';
    case GatewayStatus.ACTIVE:
      return 'Active';
    case GatewayStatus.FROZEN:
    case GatewayStatus.REJECTED:
    case GatewayStatus.REVOKED:
      return 'Attention';
    case GatewayStatus.ERROR:
      return 'Error';
    case GatewayStatus.PROOF_OF_WALLET_OWNERSHIP:
      return 'Confirm';
    case GatewayStatus.LOCATION_NOT_SUPPORTED:
    case GatewayStatus.VPN_NOT_SUPPORTED:
      return 'Not supported';
    case GatewayStatus.COLLECTING_USER_INFORMATION:
    case GatewayStatus.USER_INFORMATION_VALIDATED:
      return 'Resume';
    case GatewayStatus.VALIDATING_USER_INFORMATION:
      return 'Review';
    case GatewayStatus.USER_INFORMATION_REJECTED:
      return 'Failed';
    default:
      return 'Credential Not Found';
  }
};

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
    const passPortalUrl = `https://icp-pass-portal-l79vhikf3-civic.vercel.app?redirect_uri=${encodedRedirectUri}`;
    window.location.href = passPortalUrl;
  };

  const isActive = gatewayStatus === GatewayStatus.ACTIVE || urlCode === 'ACTIVE';

  return (
    <>
      {children}
      {isLoggedIn && (
        <div>
          <p>Gateway Status: {!isActive ? getStatusText(gatewayStatus) : 'ACTIVE'}</p>
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
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [principal, setPrincipal] = useState<Principal | undefined>(undefined);
  const [credentialService, setCredentialService] = useState<CredentialService>();
  const icpWallet = useMemo(() => ({ principal: principal?.toText() ?? undefined }), [principal]);
  const gatekeeperNetwork = "tunQheuPpHhjjsbrUDp4rikqYez9UXv4SXLRHf9Kzsv";

  const [urlCode, setUrlCode] = useState<string | null>(null);

  useEffect(() => {
    
    // Parse the URL for the 'code' parameter
    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get('code');
    setUrlCode(code);
  }, [window.location.search]);

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
      <GatewayProvider wallet={icpWallet} gatekeeperNetwork={gatekeeperNetwork} stage="dev">
        <GatewayStatusLookup isLoggedIn={isLoggedIn || urlCode === 'ACTIVE'}>
          <img src="/logo2.svg" alt="DFINITY logo" />
          {isLoggedIn && <h1>Welcome to the ICP Relying Canister</h1>}
          {isLoggedIn && <p>Logged in as {principal?.toText()}</p>}
          {/* {isLoggedIn && <button onClick={retrieveCredential}>Retrieve Credential</button>} */}
          {!isLoggedIn && urlCode !== 'ACTIVE' && <button onClick={handleLogin}>Login</button>}
        </GatewayStatusLookup>
      </GatewayProvider>
    </main>
  );
}

export default App;
