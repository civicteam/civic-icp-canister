/* A simple civic_canister that authenticates the user with Internet Identity and that
 * then issues a credential against the user principal.
 */

import { HttpAgent } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";
import type { Principal } from "@dfinity/principal";
import {  createActor, CreateActorOptions } from "../../declarations/civic_canister_backend/index";
import {_SERVICE} from "../../declarations/civic_canister_backend/civic_canister_backend.did"

const canisterId = "bkyz2-fmaaa-aaaaa-qaaaq-cai" //hardcoded civic canister id
// get it using dfx canister id civic_canister_backend
// process.env.CIVIC_CANISTER_BACKEND_ID;

// The <canisterId>.localhost URL is used as opposed to setting the canister id as a parameter
// since the latter is brittle with regards to transitively loaded resources.
const local_ii_url = `http://${process.env.INTERNET_IDENTITY_CANISTER_ID}.localhost:4943`;

let principal: Principal | undefined;
let authClient: AuthClient;

// Autofills the <input> for the II Url to point to the correct canister.
document.body.onload = () => {
  let iiUrl;

  if (process.env.DFX_NETWORK === "local") {
    iiUrl = local_ii_url;
  } else if (process.env.DFX_NETWORK === "ic") {
    iiUrl = `https://${process.env.INTERNET_IDENTITY_CANISTER_ID}.ic0.app`;
  } else {
    // fall back to local
    iiUrl = local_ii_url;
  }
  document.querySelector<HTMLInputElement>("#iiUrl")!.value = iiUrl;
};

document.getElementById("loginBtn")?.addEventListener("click", async () => {
  // When the user clicks, we start the login process.
  // First we have to create and AuthClient.
  authClient = await AuthClient.create();

  // Find out which URL should be used for login.
  const iiUrl = document.querySelector<HTMLInputElement>("#iiUrl")!.value;
  // Call authClient.login(...) to login with Internet Identity. This will open a new tab
  // with the login prompt. The code has to wait for the login process to complete.
  // We can either use the callback functions directly or wrap in a promise.
  await new Promise<void>((resolve, reject) => {
    authClient.login({
      identityProvider: iiUrl,
      onSuccess: resolve,
      onError: reject,
    });
  });

  // At this point we're authenticated, and we can get the identity from the auth client:
  const identity = authClient.getIdentity();
  principal = identity.getPrincipal();
  
  // show principal and credential button 
  document.getElementById("credentialBtn")!.style.display = 'inline';
  document.getElementById("loginStatus")!.innerText = "User Principal from Relying Party Canister POV: " + principal.toText();


});

document.getElementById("credentialBtn")?.addEventListener("click", async () => {
  // Open the II window 
  // const vcUrl = document.querySelector<HTMLInputElement>("#iiUrl")!.value + '/vc-flow';
  const vcUrl = ""
  const iiWindow = window.open(vcUrl, "_blank");

  // Listen for the JSON-RPC notification from the II window to indicate the vc flow is ready 
  window.addEventListener('message', (event) => {

    if (event.origin !== new URL(vcUrl).origin) {
      console.log("Origin mismatch:", event.origin, "expected:", new URL(vcUrl).origin);
      return; // Security check
    }

    const data = event.data;
    if (data.jsonrpc === "2.0" && data.method === "vc-flow-ready") {
      console.log("VC Flow is ready");

    // attempt to send a credential request to the II window
      const json = {
        "id": 1,
        "jsonrpc": "2.0",
        "method": "request_credential",
        "params": {
          "issuer": {
            "origin": "http://127.0.0.1:4943/?canisterId=avqkn-guaaa-aaaaa-qaaea-cai&id=bkyz2-fmaaa-aaaaa-qaaaq-cai",
            "canisterId": "bkyz2-fmaaa-aaaaa-qaaaq-cai"
          },
          "credentialSpec": {
            "credentialType": "VerifiedAdult",
          },
          "credentialSubject": "r2hdh-fwdj2-u2l6t-hsprj-zbb2a-6xqm5-keuie-cyz53-hzv4r-5fsgv-eae",
        }
      };

      // Send the JSON-RPC request to the II window
      fetch(vcUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        // mode: 'no-cors', // This disables CORS checks but also makes the response opaque
        body: JSON.stringify(json)
      }).then(response => {
        if (!response.ok) {
          console.log(response);
          throw new Error('Network response was not ok');
        }
        return response.text(); // Using text() to inspect the raw response
      }).then(text => {
        console.log("Raw response:", text);
        try {
          const result = JSON.parse(text); // Manually parsing the text to JSON
          console.log("Parsed JSON response:", result);
        } catch (parseError) {
          console.error("Error parsing JSON:", parseError);
        }
      }).catch(error => {
        console.error("Failed to send credential request:", error);
      });    }
  });

});
