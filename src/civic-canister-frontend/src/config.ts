const isProduction = import.meta.env.NODE_ENV === 'production';

const internetIdentityCanisterId = isProduction
  ? import.meta.env.VITE_PROD_INTERNET_IDENTITY_CANISTER_ID
  : import.meta.env.VITE_LOCAL_INTERNET_IDENTITY_CANISTER_ID;

export const civicBackendCanisterId = isProduction
  ? import.meta.env.VITE_PROD_CIVIC_BACKEND_CANISTER_ID
  : import.meta.env.VITE_LOCAL_CIVIC_BACKEND_CANISTER_ID;

export const civicFrontendCanisterId = isProduction
  ? import.meta.env.VITE_PROD_CIVIC_FRONTEND_CANISTER_ID
  : import.meta.env.VITE_LOCAL_CIVIC_FRONTEND_CANISTER_ID;

const host = isProduction
  ? import.meta.env.VITE_PROD_HOST
  : import.meta.env.VITE_LOCAL_HOST;

const internetIdentityUrl = `http://${internetIdentityCanisterId}.${host}`;
const civicBackendCanisterUrl = `http://${civicBackendCanisterId}.${host}`;
const civicFrontendCanisterUrl = `http://${civicFrontendCanisterId}.${host}`;

export const config = {
  internetIdentityUrl,
  civicBackendCanisterUrl,
  civicBackendCanisterId,
  internetIdentityCanisterId,
  civicFrontendCanisterUrl,
};
