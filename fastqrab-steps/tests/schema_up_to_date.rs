//! Guards that the committed `docs/static/schema.json` matches what the current
//! code produces. The GitHub Pages workflow does NOT build the binary — it ships
//! `docs/static/schema.json` verbatim — so the published schema is only as fresh
//! as that committed file. This test (run in the normal `cargo test` CI, where
//! the crate is compiled anyway) fails when the two drift apart, forcing a
//! regenerate-and-commit rather than silently serving a stale schema.

use std::path::Path;

/// Must match how the `fastqrab json-schema` CLI emits the file: pretty-printed
/// JSON. `generate_json_schema` in `dev/ci/doc_utils.py` writes the command's
/// stdout, which `println!` terminates with a trailing newline.
fn rendered_schema() -> String {
    let schema = fastqrab_steps::config::config_schema();
    serde_json::to_string_pretty(&schema).expect("schema serializes")
}

#[test]
fn committed_schema_is_up_to_date() {
    let committed_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir has a parent (the workspace root)")
        .join("docs/static/schema.json");

    let committed = std::fs::read_to_string(&committed_path).unwrap_or_else(|e| {
        panic!("could not read {}: {e}", committed_path.display());
    });

    // Compare ignoring trailing-newline differences only; everything else must
    // match byte-for-byte.
    assert_eq!(
        committed.trim_end(),
        rendered_schema().trim_end(),
        "\n{} is out of date. Regenerate and commit it:\n\
         \n    ./dev/ci/update_generated.sh\n\
         \n(or just: cargo run --bin fastqrab -- json-schema > docs/static/schema.json)\n",
        committed_path.display(),
    );
}
