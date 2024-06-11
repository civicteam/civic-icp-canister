const isProduction = import.meta.env.VITE_ENV === 'production';

console.log('isProduction:', isProduction);

const internetIdentityCanisterId = import.meta.env.VITE_INTERNET_IDENTITY_CANISTER_ID;
const civicBackendCanisterId = import.meta.env.VITE_CIVIC_BACKEND_CANISTER_ID;
const civicFrontendCanisterId = import.meta.env.VITE_CIVIC_FRONTEND_CANISTER_ID;
const host = import.meta.env.VITE_HOST;

console.log('env', { internetIdentityCanisterId, civicBackendCanisterId, civicFrontendCanisterId, host }, import.meta.env.VITE_ENV);

const internetIdentityUrl = isProduction
  ? 'https://identity.ic0.app'
  : `http://${internetIdentityCanisterId}.${host}`;

const civicBackendCanisterUrl = isProduction
  ? `https://${civicBackendCanisterId}.ic0.app`
  : `http://${civicBackendCanisterId}.${host}`;

const civicFrontendCanisterUrl = isProduction
  ? `https://${civicFrontendCanisterId}.ic0.app` 
  : `http://${civicFrontendCanisterId}.${host}`;

// This is for demo purposes but should be replaced with a more secure method
const dummyCivicSampleKey = new Uint8Array([
  73, 186, 183, 223, 243, 86, 48, 148, 83, 221, 41, 75, 229, 70, 56, 65, 247,
  179, 125, 33, 172, 58, 152, 14, 160, 114, 17, 22, 118, 0, 41, 243,
]);

export const config = {
  internetIdentityUrl,
  civicBackendCanisterUrl,
  civicBackendCanisterId,
  internetIdentityCanisterId,
  civicFrontendCanisterUrl,
  dummyCivicSampleKey,
};
