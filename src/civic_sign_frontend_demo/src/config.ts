const isProduction = import.meta.env.VITE_ENV === 'production';

console.log('isProduction:', isProduction);

const internetIdentityCanisterId = import.meta.env.VITE_INTERNET_IDENTITY_CANISTER_ID;
const host = import.meta.env.VITE_HOST;

const civicCanisterBackendUrl = "https://73ncn-4qaaa-aaaag-alddq-cai.icp0.io";

console.log('env', { internetIdentityCanisterId, host }, import.meta.env.VITE_ENV);

const internetIdentityUrl = isProduction
  ? 'https://identity.ic0.app'
  : `http://${internetIdentityCanisterId}.${host}`;

export const config = {
  internetIdentityUrl,
  civicCanisterBackendUrl,
  internetIdentityCanisterId,
};
