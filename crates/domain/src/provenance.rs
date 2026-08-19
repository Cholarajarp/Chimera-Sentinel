//! Data provenance classifications.
//!
//! Every datum displayed in the UI, stored in databases, or returned across
//! trust boundaries must be explicitly categorized by provenance.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Strict provenance classifications.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Provenance {
    /// Produced by a successful current managed/live system call and linked to real evidence.
    Live,
    /// Immutable prior evidence rendered or rerun without current side effects.
    Replay,
    /// Synthetic curated scenario for system tests.
    SystemTest,
    /// Model or statistical interpretation that is not a direct factual observation.
    Inferred,
    /// Local adapter or emulator behavior, never proof of a live managed integration.
    #[default]
    Local,
}

impl Provenance {
    pub fn is_live(&self) -> bool {
        matches!(self, Self::Live)
    }

    pub fn is_acceptable_for_production(&self) -> bool {
        matches!(self, Self::Live)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Live => "LIVE",
            Self::Replay => "REPLAY",
            Self::SystemTest => "SYSTEM_TEST",
            Self::Inferred => "INFERRED",
            Self::Local => "LOCAL",
        }
    }
}

impl fmt::Display for Provenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
