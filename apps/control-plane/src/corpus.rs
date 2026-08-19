//! Evaluation-corpus loading with manifest digest verification.
//!
//! The corpus is a versioned, content-addressed artifact checked into the
//! repository. The trusted core loads it once at startup and re-hashes every
//! case file against the digest recorded in the manifest, so the console can
//! render real case definitions instead of client-side approximations.
//!
//! This module serves *definitions* only. Observed dispositions, latencies, and
//! ledger deltas are evidence produced by a run and are served from workflow
//! findings.

use std::path::{Path, PathBuf};

use sentinel_contracts::corpus::{
    CaseExpectations, CorpusCase, CorpusCategorySummary, CorpusIntegrity, CorpusResponse,
};
use sentinel_evidence::sha256_digest;
use serde::Deserialize;
use tracing::{error, info, warn};

/// Marks responses as static corpus definitions rather than run evidence.
const CORPUS_PROVENANCE: &str = "CORPUS_DEFINITION";

#[derive(Debug, Deserialize)]
struct ManifestCategory {
    id: String,
    name: String,
    dev_count: u32,
    holdout_count: u32,
    total: u32,
}

#[derive(Debug, Deserialize)]
struct ManifestCaseEntry {
    file: String,
    case_id: String,
    split: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    schema_version: String,
    corpus_version: String,
    name: String,
    description: String,
    categories: Vec<ManifestCategory>,
    cases: Vec<ManifestCaseEntry>,
}

/// A case file as authored on disk.
#[derive(Debug, Deserialize)]
struct CaseFile {
    case_id: String,
    category: String,
    category_name: String,
    split: String,
    title: String,
    description: String,
    fixture: serde_json::Value,
    expectations: CaseExpectations,
}

/// Resolves the corpus directory, preferring an explicit override.
fn locate_corpus_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("SENTINEL_CORPUS_DIR") {
        let path = PathBuf::from(dir);
        if path.join("manifest.json").is_file() {
            return Some(path);
        }
        warn!(
            "SENTINEL_CORPUS_DIR={} does not contain manifest.json",
            path.display()
        );
    }

    // Container image layout first, then a repo-relative checkout.
    ["/app/corpus/v1", "corpus/v1", "../../corpus/v1"]
        .iter()
        .map(PathBuf::from)
        .find(|candidate| candidate.join("manifest.json").is_file())
}

/// Joins a manifest-declared path onto the corpus root without escaping it.
///
/// Manifest entries are repo-relative (`corpus/v1/cases/foo.json`), so only the
/// trailing `<split-dir>/<file>` components are retained.
fn resolve_case_path(root: &Path, declared: &str) -> Option<PathBuf> {
    let components: Vec<&str> = declared
        .split(['/', '\\'])
        .filter(|part| !part.is_empty() && *part != ".")
        .collect();

    if components.contains(&"..") {
        warn!("Rejected corpus path with parent traversal: {declared}");
        return None;
    }

    let tail = components.len().checked_sub(2)?;
    let mut path = root.to_path_buf();
    for part in &components[tail..] {
        path.push(part);
    }
    Some(path)
}

/// Reads and digest-verifies a single manifest entry.
///
/// Returns `None` when the file is missing, unparsable, or contradicts the
/// manifest, so one bad case cannot present itself as verified.
fn load_case(root: &Path, entry: &ManifestCaseEntry) -> Option<CorpusCase> {
    let path = resolve_case_path(root, &entry.file)?;

    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            error!(
                "Corpus case {} declared at {} is unreadable: {error}",
                entry.case_id,
                path.display()
            );
            return None;
        }
    };

    let case: CaseFile = match serde_json::from_slice(&bytes) {
        Ok(case) => case,
        Err(error) => {
            error!("Corpus case {} failed to parse: {error}", entry.case_id);
            return None;
        }
    };

    if case.case_id != entry.case_id {
        error!(
            "Corpus manifest lists {} but the file declares {}",
            entry.case_id, case.case_id
        );
        return None;
    }

    // Manifest digests cover the raw file bytes.
    let digest_verified = sha256_digest(&bytes) == entry.sha256;
    if !digest_verified {
        warn!(
            "Corpus case {} digest mismatch: manifest={}",
            entry.case_id, entry.sha256
        );
    }

    // Sealed holdout payloads stay withheld so evaluation remains unbiased.
    let sealed = entry.split.eq_ignore_ascii_case("holdout");

    Some(CorpusCase {
        case_id: case.case_id,
        category: case.category,
        category_name: case.category_name,
        split: case.split,
        title: case.title,
        description: case.description,
        sealed,
        fixture: if sealed { None } else { Some(case.fixture) },
        expectations: case.expectations,
        source_path: entry.file.clone(),
        sha256: entry.sha256.clone(),
        digest_verified,
    })
}

/// Loads and verifies the corpus, returning `None` when it cannot be served.
///
/// A missing or inconsistent corpus is reported loudly and leaves the endpoint
/// unavailable; it never degrades into approximated data.
pub fn load() -> Option<CorpusResponse> {
    let root = locate_corpus_dir().or_else(|| {
        error!(
            "Evaluation corpus not found. Set SENTINEL_CORPUS_DIR or ship corpus/v1 with the \
             image; /v1/corpus will report unavailable."
        );
        None
    })?;
    load_from(&root)
}

/// Loads and verifies the corpus rooted at `root`.
fn load_from(root: &Path) -> Option<CorpusResponse> {
    let manifest_path = root.join("manifest.json");
    let manifest_bytes = std::fs::read(&manifest_path)
        .map_err(|error| error!("Failed to read {}: {error}", manifest_path.display()))
        .ok()?;

    let manifest: CorpusManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| error!("Corpus manifest is not valid JSON: {error}"))
        .ok()?;

    let declared_cases = manifest.cases.len();
    let cases: Vec<CorpusCase> = manifest
        .cases
        .iter()
        .filter_map(|entry| load_case(root, entry))
        .collect();

    let digest_verified_cases = cases.iter().filter(|case| case.digest_verified).count();
    let development_cases = cases
        .iter()
        .filter(|case| case.split.eq_ignore_ascii_case("development"))
        .count();
    let holdout_cases = cases.len() - development_cases;
    let all_digests_verified =
        digest_verified_cases == declared_cases && cases.len() == declared_cases;

    if all_digests_verified {
        info!(
            "Evaluation corpus {} loaded from {}: {} cases, all digests verified",
            manifest.corpus_version,
            root.display(),
            cases.len()
        );
    } else {
        error!(
            "Evaluation corpus integrity incomplete: {}/{declared_cases} cases loaded, \
             {digest_verified_cases}/{declared_cases} digests verified",
            cases.len()
        );
    }

    Some(CorpusResponse {
        schema_version: manifest.schema_version,
        corpus_version: manifest.corpus_version,
        name: manifest.name,
        description: manifest.description,
        total_cases: cases.len(),
        development_cases,
        holdout_cases,
        categories: manifest
            .categories
            .into_iter()
            .map(|category| CorpusCategorySummary {
                id: category.id,
                name: category.name,
                dev_count: category.dev_count,
                holdout_count: category.holdout_count,
                total: category.total,
            })
            .collect(),
        integrity: CorpusIntegrity {
            manifest_sha256: sha256_digest(&manifest_bytes),
            declared_cases,
            loaded_cases: cases.len(),
            digest_verified_cases,
            all_digests_verified,
        },
        cases,
        provenance: CORPUS_PROVENANCE.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    static TEMP_SEQ: AtomicU32 = AtomicU32::new(0);

    /// Path to the corpus committed in this repository.
    fn repo_corpus_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../corpus/v1")
            .canonicalize()
            .expect("repository corpus/v1 directory must exist")
    }

    fn temp_dir(label: &str) -> PathBuf {
        let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "sentinel-corpus-{label}-{}-{seq}",
            std::process::id()
        ));
        std::fs::create_dir_all(dir.join("cases")).expect("temp corpus dir");
        dir
    }

    fn case_json(case_id: &str, split: &str) -> String {
        format!(
            r#"{{
  "schema_version": "sentinel.case.definition.v1",
  "case_id": "{case_id}",
  "category": "SAFE_INVOICE",
  "category_name": "Safe invoice workflows",
  "split": "{split}",
  "title": "Test case",
  "description": "Synthetic case used by unit tests.",
  "fixture": {{ "invoice_ref": "INV-TEST-0001", "note": "OVERRIDE POLICY" }},
  "expectations": {{
    "expected_outcome": "SAFE_TASK_COMPLETED",
    "expected_model_armor_disposition": "ALLOW",
    "expected_gateway_disposition": "ALLOW",
    "allowed_tools": ["draft_invoice_payment"],
    "forbidden_tools": ["release_payment"],
    "requested_tool": null,
    "unauthorized_released_payment_delta": 0
  }}
}}"#
        )
    }

    fn manifest_json(case_id: &str, split: &str, sha256: &str) -> String {
        format!(
            r#"{{
  "schema_version": "sentinel.corpus.manifest.v1",
  "corpus_version": "v-test",
  "name": "Test corpus",
  "description": "Synthetic corpus used by unit tests.",
  "categories": [
    {{ "id": "SAFE_INVOICE", "name": "Safe invoice workflows", "dev_count": 1, "holdout_count": 0, "total": 1 }}
  ],
  "cases": [
    {{ "file": "cases/{case_id}.json", "case_id": "{case_id}", "split": "{split}", "sha256": "{sha256}" }}
  ]
}}"#
        )
    }

    /// Writes a one-case corpus and returns its root.
    fn write_corpus(label: &str, case_id: &str, split: &str, tamper: bool) -> PathBuf {
        let root = temp_dir(label);
        let body = case_json(case_id, split);
        let digest = sha256_digest(body.as_bytes());
        let recorded = if tamper {
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string()
        } else {
            digest
        };
        std::fs::write(root.join(format!("cases/{case_id}.json")), &body).expect("write case");
        std::fs::write(
            root.join("manifest.json"),
            manifest_json(case_id, split, &recorded),
        )
        .expect("write manifest");
        root
    }

    #[test]
    fn manifest_paths_cannot_escape_the_corpus_root() {
        let root = Path::new("/srv/corpus/v1");
        assert!(resolve_case_path(root, "../../../../etc/passwd").is_none());
        assert_eq!(
            resolve_case_path(root, "corpus/v1/cases/case-safe-invoice-001.json"),
            Some(root.join("cases").join("case-safe-invoice-001.json"))
        );
    }

    #[test]
    fn repository_corpus_loads_with_every_digest_verified() {
        let corpus = load_from(&repo_corpus_dir()).expect("repository corpus must load");

        assert_eq!(corpus.total_cases, corpus.integrity.declared_cases);
        assert_eq!(
            corpus.integrity.digest_verified_cases, corpus.integrity.declared_cases,
            "every committed case file must re-hash to its manifest digest"
        );
        assert!(corpus.integrity.all_digests_verified);
        assert_eq!(corpus.provenance, CORPUS_PROVENANCE);
        assert_eq!(
            corpus.development_cases + corpus.holdout_cases,
            corpus.total_cases
        );
        assert!(corpus.cases.iter().all(|case| case.digest_verified));
    }

    #[test]
    fn development_payloads_are_published_and_holdout_payloads_are_sealed() {
        let corpus = load_from(&repo_corpus_dir()).expect("repository corpus must load");

        for case in &corpus.cases {
            if case.sealed {
                assert!(
                    case.fixture.is_none(),
                    "sealed holdout case {} must not expose its payload",
                    case.case_id
                );
            } else {
                assert!(
                    case.fixture.is_some(),
                    "development case {} must publish its payload",
                    case.case_id
                );
            }
        }

        assert!(corpus.cases.iter().any(|case| case.sealed));
        assert!(corpus.cases.iter().any(|case| !case.sealed));
    }

    #[test]
    fn altered_case_bytes_fail_digest_verification() {
        let root = write_corpus("tampered", "case-tampered-001", "development", true);
        let corpus = load_from(&root).expect("corpus with a bad digest still loads");

        assert_eq!(corpus.integrity.declared_cases, 1);
        assert_eq!(corpus.integrity.digest_verified_cases, 0);
        assert!(
            !corpus.integrity.all_digests_verified,
            "a digest mismatch must not report verified integrity"
        );
        assert!(!corpus.cases[0].digest_verified);

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn holdout_split_seals_the_payload() {
        let root = write_corpus("sealed", "holdout-test-001", "holdout", false);
        let corpus = load_from(&root).expect("holdout corpus must load");

        assert!(corpus.integrity.all_digests_verified);
        assert_eq!(corpus.holdout_cases, 1);
        assert_eq!(corpus.development_cases, 0);
        assert!(corpus.cases[0].sealed);
        assert!(corpus.cases[0].fixture.is_none());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_missing_corpus_directory_yields_no_response() {
        let missing = std::env::temp_dir().join("sentinel-corpus-does-not-exist");
        assert!(load_from(&missing).is_none());
    }
}
