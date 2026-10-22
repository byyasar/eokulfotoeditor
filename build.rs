use std::env;
use std::process::Command;

fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=app_icon.rc");
        println!("cargo:rerun-if-changed=app_icon.ico");

        let out_dir = env::var("OUT_DIR").unwrap();
        let out_file = format!("{}/app_icon.o", out_dir);

        let status = Command::new("windres")
            .args(&["app_icon.rc", "-O", "coff", "-o", &out_file])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("cargo:rustc-link-arg={}", out_file);
            }
            Err(e) => {
                eprintln!("cargo:warning=Failed to run windres: {}", e);
            }
            Ok(s) => {
                eprintln!("cargo:warning=windres exited with code: {:?}", s.code());
            }
        }
    }
}
