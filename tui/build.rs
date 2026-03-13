fn main() {
    cc::Build::new()
        .file("../circuit.c")
        .file("../analysis.c")
        .flag("-std=c11")
        .compile("circuit");

    // macOS: math is in libSystem; Linux needs explicit -lm
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=m");

    for f in &["../circuit.c", "../analysis.c", "../circuit.h", "../analysis.h"] {
        println!("cargo:rerun-if-changed={f}");
    }
}
