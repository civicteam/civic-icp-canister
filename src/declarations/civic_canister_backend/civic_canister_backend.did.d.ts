import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type ArgumentValue = { 'Int' : number } |
  { 'String' : string };
export type CredentialError = { 'NoCredentialsFound' : string };
export interface CredentialSpec {
  'arguments' : [] | [Array<[string, ArgumentValue]>],
  'credential_type' : string,
}
export interface GetCredentialRequest {
  'signed_id_alias' : SignedIdAlias,
  'prepared_context' : [] | [Uint8Array | number[]],
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
  'idp_canister_ids' : Array<Principal>,
  'ic_root_key_der' : Uint8Array | number[],
  'frontend_hostname' : string,
}
export interface PrepareCredentialRequest {
  'signed_id_alias' : SignedIdAlias,
  'credential_spec' : CredentialSpec,
}
export interface PreparedCredentialData {
  'prepared_context' : [] | [Uint8Array | number[]],
}
export type Result = { 'Ok' : Array<StoredCredential> } |
  { 'Err' : CredentialError };
export type Result_1 = { 'Ok' : IssuedCredentialData } |
  { 'Err' : IssueCredentialError };
export type Result_2 = { 'Ok' : PreparedCredentialData } |
  { 'Err' : IssueCredentialError };
export interface SignedIdAlias { 'credential_jws' : string }
export interface StoredCredential {
  'id' : string,
  'context' : Array<string>,
  'type_' : Array<string>,
  'issuer' : string,
}
export interface _SERVICE {
  'add_credentials' : ActorMethod<[Principal, Array<StoredCredential>], string>,
  'configure' : ActorMethod<[IssuerInit], undefined>,
  'get_all_credentials' : ActorMethod<[Principal], Result>,
  'get_credential' : ActorMethod<[GetCredentialRequest], Result_1>,
  'prepare_credential' : ActorMethod<[PrepareCredentialRequest], Result_2>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
