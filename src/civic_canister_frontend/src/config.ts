const isProduction = process.env.NODE_ENV === 'production';

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

const internetIdentityUrl = `https://identity.ic0.app`;
const civicBackendCanisterUrl = `https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io`;
const civicFrontendCanisterUrl = `https://d7wbq-nyaaa-aaaap-ahgiq-cai.icp0.io/`;

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