

pub mod utils;
pub mod credential;
pub mod consent_message;

// candid::export_service!();

// #[cfg(test)]
// mod test {
//     use crate::__export_service;
//     use candid_parser::utils::{service_equal, CandidSource};
//     use std::path::Path;

//     /// Checks candid interface type equality by making sure that the service in the did file is
//     /// a subtype of the generated interface and vice versa.
//     #[test]
//     fn check_candid_interface_compatibility() {
//         let canister_interface = __export_service();
//         service_equal(
//             CandidSource::Text(&canister_interface),
//             CandidSource::File(Path::new("civic_canister_backend.did")),
//         )
//         .unwrap_or_else(|e| {
//             panic!(
//                 "the canister code interface is not equal to the did file: {:?}",
//                 e
//             )
//         });
//     }
// }