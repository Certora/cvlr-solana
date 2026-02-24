/// TODO: move things from common (incremental commit)
mod common;
#[cfg(not(feature = "rt"))]
mod prover;
#[cfg(feature = "rt")]
mod rt;

pub use common::{
    cvlr_deserialize_nondet_accounts, cvlr_deserialize_nondet_accounts as cvlr_nondet_acc_infos,
    cvlr_new_account_info, fun_acc_infos_with_mem_layout,
};
#[cfg(feature = "rt")]
pub use rt::{init, GlobalDatabase};
