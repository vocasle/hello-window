use std::ffi::OsString;
use std::process::Command;
use std::{env, process};

fn main() {
    // println!("cargo:rerun-if-changed=src/shaders/vertex_shader.hlsl");
    // println!("cargo:rerun-if-changed=src/shaders/pixel_shader.hlsl");
    // println!("cargo:rerun-if-changed=src/shaders/shader.hlsli");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let fxc_exe_path =
        "C:/\"Program Files (x86)\"/\"Windows Kits\"/10/bin/10.0.22000.0/x64/fxc.exe";

    let mut shader_src_dir = manifest_dir;
    shader_src_dir.push("\\src\\shaders");

    let mut include_path = String::from("/I ");
    include_path.push_str(&shader_src_dir.to_str().unwrap());

    let paths = std::fs::read_dir(&shader_src_dir).unwrap();

    for path in paths {
        let entry = path.unwrap();

        if entry.path().extension().unwrap() == OsString::from("hlsli") {
            continue;
        }

        let path = entry.path().with_extension("");
        let filename = path.file_name().unwrap();
        let filename = filename.to_str().unwrap();

        let mut out_name = String::from("/Fo ");
        out_name.push_str(out_dir.to_str().unwrap());
        out_name.push_str("\\");
        out_name.push_str(&filename);
        out_name.push_str(".cso");

        let shader_model = if filename.contains("vs") {
            "/T vs_5_0"
        } else {
            "/T ps_5_0"
        };

        let path = entry.path();
        let args = vec![
            fxc_exe_path,
            &shader_model,
            "/E main",
            &include_path,
            "/Od",
            "/WX",
            "/Zi",
            &out_name,
            path.to_str().unwrap(),
        ];

        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(args.join(" "));

        for arg in cmd.get_args() {
            print!("{} ", arg.to_str().unwrap());
        }

        let out = cmd.output().expect("Failed to launch fxc.exe");

        if !out.status.success() {
            println!("Error: {}", String::from_utf8(out.stderr).unwrap());
            process::exit(out.status.code().unwrap());
        } else {
            println!("Stdout: {}", String::from_utf8(out.stdout).unwrap());
        }
    }
}
