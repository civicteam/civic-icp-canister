// src/App.jsx

import { useEffect, useState, useCallback } from "react";
import { PrincipalService } from "./service/PrincipalService";
import { CredentialService, Credential } from "./service/CredentialService";
import { config } from "./config";
import { Principal } from "@dfinity/principal";

const credential: Credential = {
  id: "credential-001",
  issuer: "https://example-issuer.com",
  context: ["https://www.w3.org/2018/credentials/v1"],
  claims: [{ claim_type: "VerifiedAdult", value: "true" }],
};

const principalService = new PrincipalService({
  identityProvider: config.internetIdentityUrl,
  derivationOrigin: config.civicFrontendCanisterUrl,
});

function App() {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [principal, setPrincipal] = useState<Principal | undefined>(undefined);

  // Function to handle login
  const handleLogin = useCallback(async () => {
    try {
      const userPrincipal = await principalService.requestPrincipal();
      if (userPrincipal) {
        setIsLoggedIn(true);
        setPrincipal(userPrincipal);
      }
    } catch (error) {
      console.error("Error logging in:", error);
    }
  }, []);

  const storeCredential = useCallback(async () => {
    if (!principal) {
      console.error("Principal not found");
      return;
    }

    const { civicBackendCanisterId, civicBackendCanisterUrl  } = config;
    const credentialService = new CredentialService({
      civicBackendCanisterId,
      civicBackendCanisterUrl,
    });

    await credentialService.addCredential(principal, credential);
  }, [principal]);

  // Effect to log changes
  useEffect(() => {
    console.log("Principal updated:", principal);
  }, [principal]);


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

export default App;
