pub mod contracts;
pub mod init;
pub mod privacy;
pub mod repo;

pub use contracts::*;
pub use init::{
    run_init, run_setup_check, AgentAdapter, InitAction, InitOptions, InitReport, SetupCheckItem,
    SetupCheckReport, SetupCheckStatus,
};
pub use privacy::PrivacyStatus;
pub use repo::{FileRole, RepoRoot};
