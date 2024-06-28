const isProduction = import.meta.env.VITE_ENV === 'production';

console.log('isProduction:', isProduction);

const internetIdentityCanisterId = import.meta.env.VITE_INTERNET_IDENTITY_CANISTER_ID;
const civicFrontendCanisterId = import.meta.env.VITE_CIVIC_FRONTEND_CANISTER_ID;
const host = import.meta.env.VITE_HOST;

console.log(
  "env",
  { internetIdentityCanisterId, host },
  import.meta.env.VITE_ENV
);

const civicFrontendCanisterUrl =
  "http://br5f7-7uaaa-aaaaa-qaaca-cai.localhost:4943";
// isProduction
//   ? `https://${civicFrontendCanisterId}.icp0.io` // consistently use the icp0.io domain, don't use the ic0.app domain
//   : `http://${civicFrontendCanisterId}.${host}`;

const internetIdentityUrl = isProduction
  ? "https://identity.ic0.app"
  : `http://${internetIdentityCanisterId}.${host}`;

export const config = {
  internetIdentityUrl,
  internetIdentityCanisterId,
  civicFrontendCanisterUrl,
};
