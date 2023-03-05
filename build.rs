use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::{env, process};

fn main() {
    println!("cargo:rerun-if-changed=src/shaders/vertex_shader.hlsl");
    println!("cargo:rerun-if-changed=src/shaders/pixel_shader.hlsl");
    println!("cargo:rerun-if-changed=src/shaders/shader.hlsli");

    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let profile = env::var_os("PROFILE").unwrap();
    let out_dir = PathBuf::from(&manifest_dir).join("target").join(profile);
    let shader_src_dir = PathBuf::from(&manifest_dir).join("src").join("shaders");
    let fxc_exe_path =
        "C:/\"Program Files (x86)\"/\"Windows Kits\"/10/bin/10.0.22000.0/x64/fxc.exe";
    let include_path = vec!["/I", &shader_src_dir.to_str().unwrap()].join(" ");

    let paths = std::fs::read_dir(&shader_src_dir).unwrap();
    for path in paths {
        let entry = path.unwrap();

        let is_header = match entry.path().extension() {
            Some(ext) => ext == OsString::from("hlsli"),
            None => true,
        };

        if is_header {
            continue;
        }
        let original_path = entry.path();
        let path = entry.path().with_extension("cso");
        let out_filename = PathBuf::from(&out_dir).join(path.file_name().unwrap());
        let out_name = vec!["/Fo", out_filename.to_str().unwrap()].join(" ");

        let shader_model = if original_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains("vs.hlsl")
        {
            "/T vs_5_0"
        } else {
            "/T ps_5_0"
        };

        let args = vec![
            fxc_exe_path,
            &shader_model,
            "/E main",
            &include_path,
            "/Od",
            "/WX",
            "/Zi",
            &out_name,
            original_path.to_str().unwrap(),
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
