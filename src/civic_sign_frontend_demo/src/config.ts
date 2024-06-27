const isProduction = import.meta.env.VITE_ENV === 'production';

console.log('isProduction:', isProduction);

const internetIdentityCanisterId = import.meta.env.VITE_INTERNET_IDENTITY_CANISTER_ID;
const civicBackendCanisterId = import.meta.env.VITE_CIVIC_BACKEND_CANISTER_ID;
const host = import.meta.env.VITE_HOST;

console.log('env', { internetIdentityCanisterId, host }, import.meta.env.VITE_ENV);

const civicBackendCanisterUrl = isProduction
  ? `https://${civicBackendCanisterId}.icp0.io` // consistently use the icp0.io domain, don't use the ic0.app domain
  : `http://${civicBackendCanisterId}.${host}`;

const internetIdentityUrl = isProduction
  ? "https://identity.ic0.app"
  : `http://${internetIdentityCanisterId}.${host}`;

export const config = {
  internetIdentityUrl,
  internetIdentityCanisterId,
  civicBackendCanisterUrl,
};
