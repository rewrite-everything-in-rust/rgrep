use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_basic_match() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "Hello, world!").unwrap();
    writeln!(file, "Rust is great.").unwrap();

    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("Hello")
        .arg(file_path.to_str().unwrap())
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Hello, world!"));
    assert!(!stdout.contains("Rust is great."));
}

#[test]
fn test_ignore_case() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "HELLO world").unwrap();

    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("-i")
        .arg("hello")
        .arg(file_path.to_str().unwrap())
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("HELLO world"));
}

#[test]
fn test_invert_match() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "match this").unwrap();
    writeln!(file, "exclude this").unwrap();

    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("-v")
        .arg("exclude")
        .arg(file_path.to_str().unwrap())
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("match this"));
    assert!(!stdout.contains("exclude this"));
}

#[test]
fn test_line_number() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "first line").unwrap();
    writeln!(file, "second line").unwrap();

    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("-n")
        .arg("second")
        .arg(file_path.to_str().unwrap())
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("2:second line"));
}

#[test]
fn test_count() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "rust").unwrap();
    writeln!(file, "rust").unwrap();
    writeln!(file, "cpp").unwrap();

    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("-c")
        .arg("rust")
        .arg(file_path.to_str().unwrap())
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "2");
}

#[test]
fn test_no_match() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "nothing here").unwrap();

    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("missing")
        .arg(file_path.to_str().unwrap())
        .output()
        .expect("failed to execute process");

    assert!(!output.status.success());
}
