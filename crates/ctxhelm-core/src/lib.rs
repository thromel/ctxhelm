pub mod contracts;
pub mod init;
pub mod privacy;
pub mod query_paths;
pub mod repo;

pub use contracts::*;
pub use init::{
    build_setup_run_report, claude_setup_planned_files, project_mcp_report,
    repo_setup_planned_files, run_init, run_setup_check, AgentAdapter, InitAction, InitOptions,
    InitReport, ProjectMcpAction, ProjectMcpReport, SetupCheckItem, SetupCheckReport,
    SetupCheckStatus, SetupPrivacyStatus, SetupRunReport, SetupRunReportInput,
};
pub use privacy::PrivacyStatus;
pub use query_paths::{
    explicit_query_paths, looks_like_explicit_query_path, normalize_query_path_token,
};
pub use repo::{FileRole, RepoRoot};
