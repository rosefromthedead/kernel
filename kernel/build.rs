use std::path::PathBuf;

fn main() {
    let target_triple = std::env::var("TARGET").unwrap();
    let target_arch = target_triple.split_once("-").unwrap().0;
    println!("cargo:rerun-if-changed=src/arch/{}/asm", target_arch);

    cc::Build::new()
        .files(std::fs::read_dir::<PathBuf>(
                format!("src/arch/{}/asm/", target_arch).into())
                .unwrap()
                .filter_map(Result::ok)
                .map(|dir_entry| dir_entry.path())
            )
        .compile("kernel_asm");
}
