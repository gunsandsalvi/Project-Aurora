//! ADR-0003's claim, tested: a crate that is not a dependency cannot be named.
//!
//! `crates/kernel/src/layer_probe.rs` names `aurora_world`, which is not among `kernel`'s
//! dependencies. Building it must fail, and must fail for *that* reason — an unresolved crate —
//! rather than for any other, or the fixture would pass while proving nothing.

use std::process::Command;

#[test]
fn kernel_cannot_name_world() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the tools crate always has a workspace parent")
        .to_path_buf();

    let out = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args([
            "build",
            "-p",
            "aurora-kernel",
            "--target-dir",
            "target/layer-probe",
        ])
        .env("RUSTFLAGS", "--cfg aurora_layer_probe")
        .output()
        .expect("cargo is on PATH inside a cargo test");

    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        !out.status.success(),
        "kernel named `world` and the build SUCCEEDED. The layer matrix is not being enforced.\n{stderr}"
    );
    assert!(
        stderr.contains("E0432")
            || stderr.contains("unresolved import")
            || stderr.contains("failed to resolve"),
        "the build failed, but not because `world` was unreachable from `kernel`. \
         A fixture that fails for the wrong reason proves nothing.\n{stderr}"
    );
}
