#[test]
fn shared_models_in_sync() {
    let sibling = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/gcp-log/src/models.rs");

    if !sibling.exists() {
        return; // not in workspace (e.g. published crate) — skip
    }

    fn extract(s: &str) -> &str {
        const BEGIN: &str = "// BEGIN SHARED MODELS";
        const END: &str = "// END SHARED MODELS";
        let start = s.find(BEGIN).expect("BEGIN SHARED MODELS marker missing");
        let end = s.find(END).expect("END SHARED MODELS marker missing");
        s[start..end + END.len()].trim()
    }

    let formatter = include_str!("../src/models.rs");
    let gcp_log = std::fs::read_to_string(&sibling).expect("reading gcp-log models.rs");

    assert_eq!(
        extract(formatter),
        extract(&gcp_log),
        "shared models in tracing-gcp-formatter and gcp-log have diverged — update both files"
    );
}
