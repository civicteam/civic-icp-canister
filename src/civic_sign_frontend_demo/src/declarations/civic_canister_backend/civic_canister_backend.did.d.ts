import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type ArgumentValue = { 'Int' : number } |
  { 'String' : string };
export interface Claim { 'claims' : Array<[string, ClaimValue]> }
export type ClaimValue = { 'Date' : string } |
  { 'Text' : string } |
  { 'Boolean' : boolean } |
  { 'Number' : bigint } |
  { 'Claim' : Claim };
export interface Credential {
  'id' : string,
  'context' : Array<string>,
  'type_' : Array<string>,
  'claim' : Array<Claim>,
}
export type CredentialError = { 'UnauthorizedSubject' : string } |
  { 'NoCredentialsFound' : string };
export interface CredentialSpec {
  'arguments' : [] | [Array<[string, ArgumentValue]>],
  'credential_type' : string,
}
export interface DerivationOriginData { 'origin' : string }
export type DerivationOriginError = { 'Internal' : string } |
  { 'UnsupportedOrigin' : string };
export interface DerivationOriginRequest { 'frontend_hostname' : string }
export interface FullCredential {
  'id' : string,
  'context' : Array<string>,
  'type_' : Array<string>,
  'claim' : Array<Claim>,
  'issuer' : string,
}
export interface GetCredentialRequest {
  'signed_id_alias' : SignedIdAlias,
  'prepared_context' : [] | [Uint8Array | number[]],
  'credential_spec' : CredentialSpec,
}
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
  'certificate_version' : [] | [number],
}
export interface HttpResponse {
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
  'status_code' : number,
}
export interface Icrc21ConsentInfo {
  'consent_message' : string,
  'language' : string,
}
export interface Icrc21ConsentPreferences { 'language' : string }
export type Icrc21Error = {
    'GenericError' : { 'description' : string, 'error_code' : bigint }
  } |
  { 'UnsupportedCanisterCall' : Icrc21ErrorInfo } |
  { 'ConsentMessageUnavailable' : Icrc21ErrorInfo };
export interface Icrc21ErrorInfo { 'description' : string }
export interface Icrc21VcConsentMessageRequest {
  'preferences' : Icrc21ConsentPreferences,
  'credential_spec' : CredentialSpec,
}
export type IssueCredentialError = { 'Internal' : string } |
  { 'SignatureNotFound' : string } |
  { 'InvalidIdAlias' : string } |
  { 'UnauthorizedSubject' : string } |
  { 'UnknownSubject' : string } |
  { 'UnsupportedCredentialSpec' : string };
export interface IssuedCredentialData { 'vc_jws' : string }
export interface IssuerInit {
  'derivation_origin' : string,
  'admin' : Principal,
  'idp_canister_ids' : Array<Principal>,
  'ic_root_key_der' : Uint8Array | number[],
  'authorized_issuers' : Array<Principal>,
  'frontend_hostname' : string,
}
export interface PrepareCredentialRequest {
  'signed_id_alias' : SignedIdAlias,
  'credential_spec' : CredentialSpec,
}
export interface PreparedCredentialData {
  'prepared_context' : [] | [Uint8Array | number[]],
}
export type Result = { 'Ok' : string } |
  { 'Err' : CredentialError };
export type Result_1 = { 'Ok' : DerivationOriginData } |
  { 'Err' : DerivationOriginError };
export type Result_2 = { 'Ok' : Array<FullCredential> } |
  { 'Err' : CredentialError };
export type Result_3 = { 'Ok' : IssuedCredentialData } |
  { 'Err' : IssueCredentialError };
export type Result_4 = { 'Ok' : PreparedCredentialData } |
  { 'Err' : IssueCredentialError };
export type Result_5 = { 'Ok' : Icrc21ConsentInfo } |
  { 'Err' : Icrc21Error };
export interface SignedIdAlias { 'credential_jws' : string }
export interface StoredCredential {
  'id' : string,
  'type_' : Array<string>,
  'claim' : Array<Claim>,
  'context_issuer_id' : u16,
}
export type u16 = number;
export interface _SERVICE {
  'add_credentials' : ActorMethod<[Principal, Array<Credential>], Result>,
  'add_issuer' : ActorMethod<[Principal], undefined>,
  'configure' : ActorMethod<[IssuerInit], undefined>,
  'derivation_origin' : ActorMethod<[DerivationOriginRequest], Result_1>,
  'get_admin' : ActorMethod<[], Principal>,
  'get_all_credentials' : ActorMethod<[Principal], Result_2>,
  'get_credential' : ActorMethod<[GetCredentialRequest], Result_3>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'prepare_credential' : ActorMethod<[PrepareCredentialRequest], Result_4>,
  'remove_credential' : ActorMethod<[Principal, string], Result>,
  'remove_issuer' : ActorMethod<[Principal], undefined>,
  'update_credential' : ActorMethod<[Principal, string, Credential], Result>,
  'vc_consent_message' : ActorMethod<[Icrc21VcConsentMessageRequest], Result_5>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
