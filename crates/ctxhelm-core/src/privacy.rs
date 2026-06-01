use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyStatus {
    pub local_only: bool,
    pub remote_embeddings_used: bool,
    pub remote_reranking_used: bool,
    pub redactions_applied: u32,
}

impl PrivacyStatus {
    pub fn local_only() -> Self {
        Self {
            local_only: true,
            remote_embeddings_used: false,
            remote_reranking_used: false,
            redactions_applied: 0,
        }
    }
}
