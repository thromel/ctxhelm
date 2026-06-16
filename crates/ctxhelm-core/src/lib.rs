pub mod contracts;
pub mod init;
pub mod privacy;
pub mod query_paths;
pub mod repo;

pub use contracts::*;
pub use init::{
    run_init, run_setup_check, AgentAdapter, InitAction, InitOptions, InitReport, SetupCheckItem,
    SetupCheckReport, SetupCheckStatus,
};
pub use privacy::PrivacyStatus;
pub use query_paths::{
    explicit_query_paths, looks_like_explicit_query_path, normalize_query_path_token,
};
pub use repo::{FileRole, RepoRoot};
