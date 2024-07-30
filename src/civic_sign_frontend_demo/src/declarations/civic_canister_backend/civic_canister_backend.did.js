export const idlFactory = ({ IDL }) => {
  const Claim = IDL.Rec();
  const IssuerInit = IDL.Record({
    'derivation_origin' : IDL.Text,
    'admin' : IDL.Principal,
    'idp_canister_ids' : IDL.Vec(IDL.Principal),
    'ic_root_key_der' : IDL.Vec(IDL.Nat8),
    'authorized_issuers' : IDL.Vec(IDL.Principal),
    'frontend_hostname' : IDL.Text,
  });
  const ClaimValue = IDL.Variant({
    'Date' : IDL.Text,
    'Text' : IDL.Text,
    'Boolean' : IDL.Bool,
    'Number' : IDL.Int64,
    'Claim' : Claim,
  });
  Claim.fill(
    IDL.Record({ 'claims' : IDL.Vec(IDL.Tuple(IDL.Text, ClaimValue)) })
  );
  const Credential = IDL.Record({
    'id' : IDL.Text,
    'context' : IDL.Vec(IDL.Text),
    'type_' : IDL.Vec(IDL.Text),
    'claim' : IDL.Vec(Claim),
  });
  const CredentialError = IDL.Variant({
    'UnauthorizedSubject' : IDL.Text,
    'NoCredentialsFound' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : CredentialError });
  const DerivationOriginRequest = IDL.Record({
    'frontend_hostname' : IDL.Text,
  });
  const DerivationOriginData = IDL.Record({ 'origin' : IDL.Text });
  const DerivationOriginError = IDL.Variant({
    'Internal' : IDL.Text,
    'UnsupportedOrigin' : IDL.Text,
  });
  const Result_1 = IDL.Variant({
    'Ok' : DerivationOriginData,
    'Err' : DerivationOriginError,
  });
  const FullCredential = IDL.Record({
    'id' : IDL.Text,
    'context' : IDL.Vec(IDL.Text),
    'type_' : IDL.Vec(IDL.Text),
    'claim' : IDL.Vec(Claim),
    'issuer' : IDL.Text,
  });
  const Result_2 = IDL.Variant({
    'Ok' : IDL.Vec(FullCredential),
    'Err' : CredentialError,
  });
  const SignedIdAlias = IDL.Record({ 'credential_jws' : IDL.Text });
  const ArgumentValue = IDL.Variant({ 'Int' : IDL.Int32, 'String' : IDL.Text });
  const CredentialSpec = IDL.Record({
    'arguments' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, ArgumentValue))),
    'credential_type' : IDL.Text,
  });
  const GetCredentialRequest = IDL.Record({
    'signed_id_alias' : SignedIdAlias,
    'prepared_context' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'credential_spec' : CredentialSpec,
  });
  const IssuedCredentialData = IDL.Record({ 'vc_jws' : IDL.Text });
  const IssueCredentialError = IDL.Variant({
    'Internal' : IDL.Text,
    'SignatureNotFound' : IDL.Text,
    'InvalidIdAlias' : IDL.Text,
    'UnauthorizedSubject' : IDL.Text,
    'UnknownSubject' : IDL.Text,
    'UnsupportedCredentialSpec' : IDL.Text,
  });
  const Result_3 = IDL.Variant({
    'Ok' : IssuedCredentialData,
    'Err' : IssueCredentialError,
  });
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'certificate_version' : IDL.Opt(IDL.Nat16),
  });
  const HttpResponse = IDL.Record({
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'status_code' : IDL.Nat16,
  });
  const PrepareCredentialRequest = IDL.Record({
    'signed_id_alias' : SignedIdAlias,
    'credential_spec' : CredentialSpec,
  });
  const PreparedCredentialData = IDL.Record({
    'prepared_context' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const Result_4 = IDL.Variant({
    'Ok' : PreparedCredentialData,
    'Err' : IssueCredentialError,
  });
  const Icrc21ConsentPreferences = IDL.Record({ 'language' : IDL.Text });
  const Icrc21VcConsentMessageRequest = IDL.Record({
    'preferences' : Icrc21ConsentPreferences,
    'credential_spec' : CredentialSpec,
  });
  const Icrc21ConsentInfo = IDL.Record({
    'consent_message' : IDL.Text,
    'language' : IDL.Text,
  });
  const Icrc21ErrorInfo = IDL.Record({ 'description' : IDL.Text });
  const Icrc21Error = IDL.Variant({
    'GenericError' : IDL.Record({
      'description' : IDL.Text,
      'error_code' : IDL.Nat,
    }),
    'UnsupportedCanisterCall' : Icrc21ErrorInfo,
    'ConsentMessageUnavailable' : Icrc21ErrorInfo,
  });
  const Result_5 = IDL.Variant({
    'Ok' : Icrc21ConsentInfo,
    'Err' : Icrc21Error,
  });
  return IDL.Service({
    'add_credentials' : IDL.Func(
        [IDL.Principal, IDL.Vec(Credential)],
        [Result],
        [],
      ),
    'add_issuer' : IDL.Func([IDL.Principal], [], []),
    'configure' : IDL.Func([IssuerInit], [], []),
    'derivation_origin' : IDL.Func([DerivationOriginRequest], [Result_1], []),
    'get_admin' : IDL.Func([], [IDL.Principal], ['query']),
    'get_all_credentials' : IDL.Func([IDL.Principal], [Result_2], ['query']),
    'get_credential' : IDL.Func([GetCredentialRequest], [Result_3], ['query']),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'prepare_credential' : IDL.Func([PrepareCredentialRequest], [Result_4], []),
    'remove_credential' : IDL.Func([IDL.Principal, IDL.Text], [Result], []),
    'remove_issuer' : IDL.Func([IDL.Principal], [], []),
    'update_credential' : IDL.Func(
        [IDL.Principal, IDL.Text, Credential],
        [Result],
        [],
      ),
    'vc_consent_message' : IDL.Func(
        [Icrc21VcConsentMessageRequest],
        [Result_5],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const IssuerInit = IDL.Record({
    'derivation_origin' : IDL.Text,
    'admin' : IDL.Principal,
    'idp_canister_ids' : IDL.Vec(IDL.Principal),
    'ic_root_key_der' : IDL.Vec(IDL.Nat8),
    'authorized_issuers' : IDL.Vec(IDL.Principal),
    'frontend_hostname' : IDL.Text,
  });
  return [IDL.Opt(IssuerInit)];
};
