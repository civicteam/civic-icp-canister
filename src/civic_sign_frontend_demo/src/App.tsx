import React, { useEffect, useState, useCallback, useMemo } from 'react';
import { Principal } from '@dfinity/principal';
import { PrincipalService } from './service/PrincipalService.js';
import { config, GatekeeperNetwork } from './config.js';
import ICPCredentialCheckButton, { CredentialCheckResponse } from '@civic/icp-gateway-react-ui';
import { ChevronDown } from 'lucide-react';

function App() {
  const [principal, setPrincipal] = useState<Principal | undefined>(undefined);
  const [urlCode, setUrlCode] = useState<string | null>(null);
  const [selectedGatekeeperNetwork, setSelectedGatekeeperNetwork] = useState<GatekeeperNetwork>(config.gatekeeperNetworks[0]);

  // Assume we have an array of available gatekeeperNetworks
  const gatekeeperNetworks = useMemo(() => config.gatekeeperNetworks, []);

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

  const handleGatekeeperNetworkChange = useCallback((event: React.ChangeEvent<HTMLSelectElement>) => {
    const selectedNetwork = gatekeeperNetworks.find((network) => network.name === event.target.value);
    console.log('Selected network:', selectedNetwork);
    setSelectedGatekeeperNetwork(selectedNetwork || gatekeeperNetworks[0]);
  }, []);

  return (
    <main className="min-h-screen bg-gray-100 flex flex-col items-center justify-center p-4">
      <div className="bg-white rounded-lg shadow-lg p-8 max-w-md w-full">
        <img src="/logo2.svg" alt="DFINITY logo" className="mx-auto mb-8 w-32" />
        
        {principal ? (
          <div className="flex flex-col items-center justify-center">
            <h1 className="text-2xl font-bold text-center mb-4">Welcome to the ICP Relying Canister</h1>
            <p className="text-center mb-6">Logged in as <span className="font-mono bg-gray-100 p-1 rounded">{principal?.toText()}</span></p>
            <ICPCredentialCheckButton
              principal={principal}
              gatekeeperNetwork={selectedGatekeeperNetwork.address}
              onCredentialCheck={handleCredentialCheck}
            />
          </div>
        ) : (
          <>
            <h2 className="text-xl font-semibold mb-4">Select Gatekeeper Network</h2>
            <div className="relative mb-6">
              <select
                id="gatekeeperNetwork"
                value={selectedGatekeeperNetwork.name}
                onChange={handleGatekeeperNetworkChange}
                className="block appearance-none w-full bg-white border border-gray-300 text-gray-700 py-3 px-4 pr-8 rounded leading-tight focus:outline-none focus:bg-white focus:border-gray-500"
              >
                {gatekeeperNetworks.map((network) => (
                  <option key={network.name} value={network.name}>
                    {network.name}
                  </option>
                ))}
              </select>
              <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-700">
                <ChevronDown size={20} />
              </div>
            </div>
            {!urlCode && (
              <button
                onClick={handleLogin}
                className="w-full bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline transition duration-300"
              >
                Login
              </button>
            )}
          </>
        )}
        
        {urlCode && (
          <div className="mt-6 text-center">
            <img
              src={'https://www.icegif.com/wp-content/uploads/2023/01/icegif-162.gif'}
              alt="Cool GIF for active status"
              className="w-24 h-24 mx-auto"
            />
            <p className="mt-2 text-sm text-gray-600">Success!</p>
          </div>
        )}
      </div>
    </main>
  );
}

export default App;