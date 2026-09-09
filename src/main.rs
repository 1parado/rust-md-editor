//! Source is stored compressed as `src/main.rs.gz.b64` and expanded by CI before build.
//! Local expand:
//!   base64 -d src/main.rs.gz.b64 | gzip -d > src/main.rs
fn main() {
    eprintln!("Expand src/main.rs.gz.b64 first (see Agent.md / CI workflow).");
}
