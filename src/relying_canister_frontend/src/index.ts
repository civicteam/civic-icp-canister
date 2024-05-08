import { requestVerifiablePresentation } from "@dfinity/verifiable-credentials/request-verifiable-presentation";
import { AuthClient } from "@dfinity/auth-client";
import type { Principal } from "@dfinity/principal";
import {_SERVICE} from "../../civic_canister_frontend/src/civic_canister_backend/civic_canister_backend.did"

// const canisterId = "bkyz2-fmaaa-aaaaa-qaaaq-cai" //hardcoded civic canister id, get it using dfx canister id civic_canister_backend

const local_ii_url = `http://${process.env.INTERNET_IDENTITY_CANISTER_ID}.localhost:4943`;

let principal: Principal;

document.body.onload = () => {
  let iiUrl;

  if (process.env.DFX_NETWORK === "local") {
    iiUrl = local_ii_url;
  } else if (process.env.DFX_NETWORK === "ic") {
    iiUrl = `https://${process.env.INTERNET_IDENTITY_CANISTER_ID}.ic0.app`;
  } else {
    iiUrl = local_ii_url;
  }
  document.querySelector<HTMLInputElement>("#iiUrl")!.value = iiUrl;
};

document.getElementById("loginBtn")?.addEventListener("click", async () => {
  const authClient = await AuthClient.create();
  const iiUrl = document.querySelector<HTMLInputElement>("#iiUrl")!.value;
  await new Promise<void>((resolve, reject) => {
    authClient.login({
      identityProvider: iiUrl,
      onSuccess: resolve,
      onError: reject,
    });
  });

  const identity = authClient.getIdentity();
  principal = identity.getPrincipal();
  
  // show principal and credential button 
  document.getElementById("credentialBtn")!.style.display = 'inline';
  document.getElementById("loginStatus")!.innerText = "User Principal from Relying Party Canister POV: " + principal.toText();
});

document.getElementById("credentialBtn")?.addEventListener("click", async () => {
  // Define the credential data
const credentialData = {
  credentialSpec: {
    credentialType: 'VerifiedAdult',
    arguments: {}
  },
  credentialSubject: principal.toText()
};

// Define the issuer data
const issuerData = {
  "origin": "http://bkyz2-fmaaa-aaaaa-qaaaq-cai.localhost:4943",
  // "canisterId": canisterId
};

// Callback functions
const onSuccess = (response: any) => {
  console.log('VC Request Successful:', response);
  displayCredential(response);
};

const onError = (error: any) => {
  console.error('VC Request Failed:', error);
};

const identityProvider = local_ii_url;

const derivationOrigin = undefined;

const requestParams = {
  onSuccess,
  onError,
  credentialData,
  issuerData,
  identityProvider,
  derivationOrigin
};

requestVerifiablePresentation(requestParams);


const displayCredential = (credential: any) => {
  // Update the DOM or state with the credential information
  document.getElementById('credentialStatus')!.textContent = JSON.stringify(credential, null, 2);
};
});
