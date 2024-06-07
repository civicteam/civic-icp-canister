import { CivicSignProveFactory } from '@civic/civic-sign';
import axios from 'axios';

const button = document.querySelector(
  "[data-button-id=authenticate]"
)! as HTMLButtonElement;
const isAuthed = document.querySelector(
  "[data-output-id=is-authed]"
)! as HTMLOutputElement;

export const uint8ArrayToHexString = (bytes: Uint8Array | number[]) => {
  if (!(bytes instanceof Uint8Array)) {
    bytes = Uint8Array.from(bytes);
  }
  return bytes.reduce(
    (str, byte) => str + byte.toString(16).padStart(2, "0"),
    ""
  );
};

const onSignChallenge = async () => {
  // CALL CIVIC-SIGN BACKEND /nonce
  // const resp = await fetch("/challenge");
  // const obj: { challenge: string } = await resp.json();
  // const challenge = obj.challenge;

  const nonce = await getNonce('dev');
  console.log(nonce);

  // CIVIC-SIGN HERE
  // const delegationIdentity = await authWithII({
  //   // The url needs to be aligned with the root key in the backend
  //   // url: "http://internet_identity.localhost:5173",
  //   url: "https://jqajs-xiaaa-aaaad-aab5q-cai.ic0.app/",
  //   sessionPublicKey: new Uint8Array(Buffer.from(challenge, "base64")),
  // });

  const civicSignProve = CivicSignProveFactory.createWithICPWallet(
    {principal: 'not known here'}, {url: 'http://aovwi-4maaa-aaaaa-qaagq-cai.localhost:4943/'});
  const proof = await civicSignProve.requestProof(nonce.toString());
  console.log(proof);

  // // SKIP FOR NOW
  // const data = { challenge, delegationIdentity };
  // await fetch("/verify", {
  //   method: "POST",
  //   body: JSON.stringify(data, (_, v) => {
  //     if (typeof v === "bigint") {
  //       // We need to expiration date to be hex string.
  //       return v.toString(16);
  //     }
  //     if (v instanceof Uint8Array) {
  //       // We need the keys to be hex strings.
  //       return uint8ArrayToHexString(v);
  //     }
  //     return v;
  //   }),
  //   headers: new Headers({
  //     "Content-Type": "application/json",
  //   }),
  // });
};
button.addEventListener("click", onSignChallenge);

button.disabled = false;

type Nonce = { nonce: string; timestamp: number };
const getNonce = async (civicPassApiStage: string): Promise<Nonce> => {
  const response = await axios.get<Nonce>(`https://dev.api.civic.com/sign-${civicPassApiStage}/nonce`);
  return response.data;
};
