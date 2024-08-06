const isDev = import.meta.env.VITE_ENV === 'development';

console.log('isDev:', isDev);

const internetIdentityCanisterId = import.meta.env.VITE_INTERNET_IDENTITY_CANISTER_ID;
const civicBackendCanisterId = import.meta.env.VITE_CIVIC_BACKEND_CANISTER_ID;
const relyingFrontendCanisterId = import.meta.env.VITE_RELYING_FRONTEND_CANISTER_ID;
const host = import.meta.env.VITE_HOST;

console.log('env', { internetIdentityCanisterId, civicBackendCanisterId, relyingFrontendCanisterId, host }, import.meta.env.VITE_ENV);

const internetIdentityUrl = !isDev
  ? 'https://identity.ic0.app'
  : `http://${internetIdentityCanisterId}.${host}`;

const civicBackendCanisterUrl = !isDev
  ? `https://${civicBackendCanisterId}.icp0.io` // consistently use the icp0.io domain, don't use the ic0.app domain
  : `http://${civicBackendCanisterId}.${host}`;

const relyingFrontendCanisterUrl = !isDev
  ? `https://${relyingFrontendCanisterId}.icp0.io`
  : `http://${relyingFrontendCanisterId}.${host}`;


const portalUrl = `https://icp-getpass.civic.com`;
const gatekeeperNetwork = "tigoYhp9SpCDoCQmXGj2im5xa3mnjR1zuXrpCJ5ZRmi";

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
  relyingFrontendCanisterUrl,
  dummyCivicSampleKey,
  portalUrl,
  gatekeeperNetwork,
};
