import { requestVerifiablePresentation } from "@dfinity/verifiable-credentials/request-verifiable-presentation";

import { HttpAgent } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";
import type { Principal } from "@dfinity/principal";
import {  createActor, CreateActorOptions } from "../../declarations/civic_canister_backend/index";
import {_SERVICE} from "../../declarations/civic_canister_backend/civic_canister_backend.did"

const canisterId = "bkyz2-fmaaa-aaaaa-qaaaq-cai" //hardcoded civic canister id
// get it using dfx canister id civic_canister_backend
// process.env.CIVIC_CANISTER_BACKEND_ID;

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
  // // Open the II window 
  // const vcUrl = document.querySelector<HTMLInputElement>("#iiUrl")!.value + '/vc-flow';
  // const iiWindow = window.open(vcUrl, "_blank");

  // // Listen for the JSON-RPC notification from the II window to indicate the vc flow is ready 
  // window.addEventListener('message', (event) => {

  //   if (event.origin !== new URL(vcUrl).origin) {
  //     console.log("Origin mismatch:", event.origin, "expected:", new URL(vcUrl).origin);
  //     return; // Security check
  //   }


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
  "origin": "http://127.0.0.1:4943/?canisterId=br5f7-7uaaa-aaaaa-qaaca-cai&id=bkyz2-fmaaa-aaaaa-qaaaq-cai",
  "canisterId": "bkyz2-fmaaa-aaaaa-qaaaq-cai"
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

requestVerifiablePresentation(onSuccess, onError, credentialData, issuerData, identityProvider);

const displayCredential = (credential) => {
  // Update the DOM or state with the credential information
  document.getElementById('credentialStatus')!.textContent = JSON.stringify(credential, null, 2);
};
    // const data = event.data;
//     if (data.jsonrpc === "2.0" && data.method === "vc-flow-ready") {
//       console.log("VC Flow is ready");

//     // attempt to send a credential request to the II window
//       const json = {
//         "id": 1,
//         "jsonrpc": "2.0",
//         "method": "request_credential",
//         "params": {
//           "issuer": {
//             "origin": "http://127.0.0.1:4943/?canisterId=avqkn-guaaa-aaaaa-qaaea-cai&id=bkyz2-fmaaa-aaaaa-qaaaq-cai",
//             "canisterId": "bkyz2-fmaaa-aaaaa-qaaaq-cai"
//           },
//           "credentialSpec": {
//             "credentialType": "VerifiedAdult",
//           },
//           "credentialSubject": "r2hdh-fwdj2-u2l6t-hsprj-zbb2a-6xqm5-keuie-cyz53-hzv4r-5fsgv-eae",
//         }
//       };

//       // Send the JSON-RPC request to the II window
//       fetch(vcUrl, {
//         method: 'POST',
//         headers: {
//           'Content-Type': 'application/json'
//         },
//         // mode: 'no-cors', // This disables CORS checks but also makes the response opaque
//         body: JSON.stringify(json)
//       }).then(response => {
//         if (!response.ok) {
//           console.log(response);
//           throw new Error('Network response was not ok');
//         }
//         return response.text(); // Using text() to inspect the raw response
//       }).then(text => {
//         console.log("Raw response:", text);
//         try {
//           const result = JSON.parse(text); // Manually parsing the text to JSON
//           console.log("Parsed JSON response:", result);
//         } catch (parseError) {
//           console.error("Error parsing JSON:", parseError);
//         }
//       }).catch(error => {
//         console.error("Failed to send credential request:", error);
//       });    }
//   });

// });
});
