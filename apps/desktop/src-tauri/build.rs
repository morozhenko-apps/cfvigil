use std::{env, path::PathBuf, process::Command};

fn execute(name: &str, args: &[&str]) -> String {
    let result = Command::new(name)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("Cannot execute {name}: {error}"));
    if !result.status.success() {
        panic!("{name} failed: {}", String::from_utf8_lossy(&result.stderr));
    }
    String::from_utf8(result.stdout)
        .unwrap_or_else(|error| panic!("{name} printed invalid UTF-8: {error}"))
        .trim()
        .to_owned()
}

fn compile_wayland_bridge() {
    println!("cargo:rerun-if-changed=native/wayland_relative_drag.c");
    let root = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let protocol_dir = execute(
        "pkg-config",
        &["--variable=pkgdatadir", "wayland-protocols"],
    );
    let protocol = PathBuf::from(protocol_dir)
        .join("unstable/relative-pointer/relative-pointer-unstable-v1.xml");
    if !protocol.exists() {
        panic!("Install wayland-protocols: missing {}", protocol.display());
    }

    let header = root.join("relative-pointer-unstable-v1-client-protocol.h");
    let glue = root.join("relative-pointer-protocol.c");
    let header_text = header.to_string_lossy();
    let glue_text = glue.to_string_lossy();
    let protocol_text = protocol.to_string_lossy();
    execute(
        "wayland-scanner",
        &["client-header", &protocol_text, &header_text],
    );
    execute(
        "wayland-scanner",
        &["private-code", &protocol_text, &glue_text],
    );

    let flags = execute(
        "pkg-config",
        &["--cflags", "gdk-wayland-3.0", "wayland-client"],
    );
    let mut objects = Vec::new();
    for (name, source) in [
        ("relative-bridge", "native/wayland_relative_drag.c"),
        ("relative-protocol", glue_text.as_ref()),
    ] {
        let object = root.join(format!("{name}.o"));
        let mut cc = Command::new("cc");
        cc.args(["-std=c11", "-fPIC", "-Wall", "-Wextra", "-Werror"]);
        cc.arg("-I").arg(&root);
        cc.args(flags.split_whitespace());
        cc.arg("-c").arg(source).arg("-o").arg(&object);
        let status = cc.status().expect("Cannot start C compiler");
        assert!(status.success(), "Cannot compile {source}");
        objects.push(object);
    }

    let archive = root.join("libcfvigil_wayland_relative.a");
    let status = Command::new("ar")
        .arg("crs")
        .arg(&archive)
        .args(&objects)
        .status()
        .expect("Cannot start archive tool");
    assert!(status.success(), "Cannot archive Wayland bridge");

    println!("cargo:rustc-link-search=native={}", root.display());
    println!("cargo:rustc-link-lib=static=cfvigil_wayland_relative");
    println!("cargo:rustc-link-lib=dylib=gdk-3");
    println!("cargo:rustc-link-lib=dylib=wayland-client");
}

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        compile_wayland_bridge();
    }
    tauri_build::build();
}
