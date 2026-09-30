use std::path::PathBuf;

/// The nine translation units that make up `libswe`, per the upstream Makefile
/// (`SWEOBJ`). Deliberately excludes the files that carry a `main()`
/// (swetest.c, swemini.c, obama.c, swevents.c) and the optional ephe4 format
/// readers (sweephe4.c, swephgen4.c).
const SOURCES: &[&str] = &[
    "swedate.c",
    "swehouse.c",
    "swejpl.c",
    "swemmoon.c",
    "swemplan.c",
    "sweph.c",
    "swephlib.c",
    "swecl.c",
    "swehel.c",
];

fn main() {
    let vendor = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor");

    let mut build = cc::Build::new();
    build
        .include(&vendor)
        // Upstream is warning-noisy C89; we are not going to patch it, and
        // patching it would complicate future upstream merges.
        .warnings(false)
        .extra_warnings(false)
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-misleading-indentation")
        // Determinism: never let the compiler contract or reassociate float
        // ops. Identical input must give bit-identical output on every target.
        .flag_if_supported("-ffp-contract=off")
        .flag_if_supported("-fno-fast-math")
        // One global Swiss Ephemeris state on every platform. Upstream makes
        // its state thread-local (`__thread`) everywhere except Apple, so on
        // Linux or Android an ephemeris path set on one thread is invisible to
        // the others. Found by the Phase 4 Linux QA run, where worker threads
        // looked in the default path. lagn-ephem serialises all access behind
        // its own mutex, so global state is safe - and it is what was
        // validated on macOS.
        .define("TLSOFF", None);

    // WebAssembly has no dynamic loader, and upstream's swe_get_library_path
    // calls dladdr to find its own .so. NO_SWE_GLP is upstream's own switch
    // for suppressing that function; nothing here calls it.
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch == "wasm32" {
        build.define("NO_SWE_GLP", None);
    }

    for src in SOURCES {
        build.file(vendor.join(src));
        println!("cargo:rerun-if-changed=vendor/{src}");
    }
    println!("cargo:rerun-if-changed=build.rs");

    build.compile("swe");

    // libm: present in libSystem on Apple platforms and in wasi-libc, separate
    // elsewhere.
    if target_arch != "wasm32" && !cfg!(target_vendor = "apple") && cfg!(unix) {
        println!("cargo:rustc-link-lib=m");
    }
}
