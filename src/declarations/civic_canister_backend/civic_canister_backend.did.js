export const idlFactory = ({ IDL }) => {
  const Claim = IDL.Rec();
  const IssuerInit = IDL.Record({
    'derivation_origin' : IDL.Text,
    'idp_canister_ids' : IDL.Vec(IDL.Principal),
    'ic_root_key_der' : IDL.Vec(IDL.Nat8),
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
  const StoredCredential = IDL.Record({
    'id' : IDL.Text,
    'context' : IDL.Vec(IDL.Text),
    'type_' : IDL.Vec(IDL.Text),
    'claim' : IDL.Vec(Claim),
    'issuer' : IDL.Text,
  });
  const DerivationOriginRequest = IDL.Record({
    'frontend_hostname' : IDL.Text,
  });
  const DerivationOriginData = IDL.Record({ 'origin' : IDL.Text });
  const DerivationOriginError = IDL.Variant({
    'Internal' : IDL.Text,
    'UnsupportedOrigin' : IDL.Text,
  });
  const Result = IDL.Variant({
    'Ok' : DerivationOriginData,
    'Err' : DerivationOriginError,
  });
  const CredentialError = IDL.Variant({ 'NoCredentialsFound' : IDL.Text });
  const Result_1 = IDL.Variant({
    'Ok' : IDL.Vec(StoredCredential),
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
  const Result_2 = IDL.Variant({
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
  const Result_3 = IDL.Variant({
    'Ok' : PreparedCredentialData,
    'Err' : IssueCredentialError,
  });
  return IDL.Service({
    'add_credentials' : IDL.Func(
        [IDL.Principal, IDL.Vec(StoredCredential)],
        [IDL.Text],
        [],
      ),
    'configure' : IDL.Func([IssuerInit], [], []),
    'derivation_origin' : IDL.Func([DerivationOriginRequest], [Result], []),
    'get_all_credentials' : IDL.Func([IDL.Principal], [Result_1], ['query']),
    'get_credential' : IDL.Func([GetCredentialRequest], [Result_2], ['query']),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'prepare_credential' : IDL.Func([PrepareCredentialRequest], [Result_3], []),
  });
};
export const init = ({ IDL }) => {
  const IssuerInit = IDL.Record({
    'derivation_origin' : IDL.Text,
    'idp_canister_ids' : IDL.Vec(IDL.Principal),
    'ic_root_key_der' : IDL.Vec(IDL.Nat8),
    'frontend_hostname' : IDL.Text,
  });
  return [IDL.Opt(IssuerInit)];
};
