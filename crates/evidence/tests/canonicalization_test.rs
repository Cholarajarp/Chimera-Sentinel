use serde_json::json;
use sentinel_evidence::{canonical_digest, canonicalize, canonicalize_value};

#[test]
fn test_rfc8785_canonicalization_key_ordering() {
    let val1 = json!({
        "z": 1,
        "a": 2,
        "m": 3
    });
    let val2 = json!({
        "a": 2,
        "m": 3,
        "z": 1
    });

    let s1 = canonicalize_value(&val1);
    let s2 = canonicalize_value(&val2);

    assert_eq!(s1, s2);
    assert_eq!(s1, r#"{"a":2,"m":3,"z":1}"#);

    let d1 = canonical_digest(&val1).unwrap();
    let d2 = canonical_digest(&val2).unwrap();
    assert_eq!(d1, d2);
    assert!(d1.starts_with("sha256:"));
}

#[test]
fn test_rfc8785_nested_canonicalization() {
    let nested = json!({
        "candidate": {
            "version": "1.0",
            "capabilities": ["draft", "status"]
        },
        "b": null,
        "active": true
    });

    let s = canonicalize_value(&nested);
    assert_eq!(s, r#"{"active":true,"b":null,"candidate":{"capabilities":["draft","status"],"version":"1.0"}}"#);
}
