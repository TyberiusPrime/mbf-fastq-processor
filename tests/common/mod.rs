use ex::fs::File;
use std::io::Write;
use tempfile::tempdir;

pub fn run(config: &str) -> tempfile::TempDir {
    let td = tempdir().unwrap();
    let config_file = td.path().join("config.toml");
    let mut f = File::create(&config_file).unwrap();
    f.write_all(config.as_bytes()).unwrap();

    let error_file = td.path().join("error");
    let _f = File::create(&error_file).unwrap();
    mbf_fastq_processor::run(&config_file, td.path()).unwrap();
    //remove the error  file again. If it's still present, we had a panic
    std::fs::remove_file(&error_file).unwrap();
    td
}

#[allow(dead_code)] // false positive?
pub fn run_and_capture(config: &str) -> (tempfile::TempDir, String, String) {
    let td = tempdir().unwrap();
    let config_file = td.path().join("config.toml");
    let mut f = File::create(&config_file).unwrap();
    f.write_all(config.as_bytes()).unwrap();

    let error_file = td.path().join("error");
    let _f = File::create(&error_file).unwrap();
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        //.join("debug")
        .join("mbf_fastq_processor");

    let cmd = std::process::Command::new(bin_path)
        .arg(&config_file)
        .arg(td.path())
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();
    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();
    if !(cmd.status.success()) {
        dbg!(&stderr);
    }
    assert!(cmd.status.success());
    //remove the error  file again. If it's still present, we had a panic
    std::fs::remove_file(&error_file).unwrap();
    (td, stdout, stderr)
}
