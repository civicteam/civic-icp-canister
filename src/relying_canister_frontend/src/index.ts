import { requestVerifiablePresentation } from "@dfinity/verifiable-credentials/request-verifiable-presentation";
import { AuthClient } from "@dfinity/auth-client";
import type { Principal } from "@dfinity/principal";
import {_SERVICE} from "../../declarations/civic_canister_backend/civic_canister_backend.did"

const canisterId = "b77ix-eeaaa-aaaaa-qaada-cai" //hardcoded civic canister id, get it using dfx canister id civic_canister_backend

const local_ii_url = `http://${process.env.INTERNET_IDENTITY_CANISTER_ID}.localhost:4943`;

let principal: Principal | undefined;
let authClient: AuthClient;

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
  authClient = await AuthClient.create();
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
  credentialSubject: 'r2hdh-fwdj2-u2l6t-hsprj-zbb2a-6xqm5-keuie-cyz53-hzv4r-5fsgv-eae'
};

// Define the issuer data
const issuerData = {
  "origin": "http://b77ix-eeaaa-aaaaa-qaada-cai.localhost:4943",
  "canisterId": canisterId
};

// Callback functions
const onSuccess = (response) => {
  console.log('VC Request Successful:', response);
  displayCredential(response);
};

const onError = (error) => {
  console.error('VC Request Failed:', error);
};

const identityProvider = local_ii_url;

const requestParams = {
  onSuccess,
  onError,
  credentialData,
  issuerData,
  identityProvider
};

requestVerifiablePresentation(requestParams);


const displayCredential = (credential) => {
  // Update the DOM or state with the credential information
  document.getElementById('credentialStatus')!.textContent = JSON.stringify(credential, null, 2);
};
});
