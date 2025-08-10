use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn get_test_video_path(filename: &str) -> std::path::PathBuf {
    Path::new("test_videos/samples").join(filename)
}

fn get_output_path(filename: &str) -> std::path::PathBuf {
    Path::new("test_videos/output").join(filename)
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("small-mp4").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Squeeze your videos for easy sharing"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("small-mp4").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("small-mp4"));
}

#[test]
fn test_cli_list_hardware() {
    let mut cmd = Command::cargo_bin("small-mp4").unwrap();
    cmd.arg("list-hw")
        .assert()
        .success()
        .stdout(predicate::str::contains("Detecting hardware acceleration"));
}

#[test]
fn test_cli_compress_basic() {
    let input_path = get_test_video_path("test_720p_10s.mp4");
    let output_path = get_output_path("cli_test_720p_compressed.mp4");

    if !input_path.exists() {
        eprintln!("Skipping test: {} not found", input_path.display());
        return;
    }

    // Ensure output directory exists
    fs::create_dir_all("test_videos/output").unwrap();

    // Remove output file if it exists
    let _ = fs::remove_file(&output_path);

    let mut cmd = Command::cargo_bin("small-mp4").unwrap();
    cmd.arg("compress")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--size")
        .arg("1mb")
        .arg("--force-software")
        .assert()
        .success();

    // Verify output was created
    assert!(output_path.exists());
    
    // Verify size is under target
    let metadata = fs::metadata(&output_path).unwrap();
    let size_mb = metadata.len() as f32 / (1024.0 * 1024.0);
    assert!(size_mb <= 1.1, "Output size {} MB exceeds target", size_mb);
}

#[test]
fn test_cli_compress_different_sizes() {
    let input_path = get_test_video_path("test_720p_10s.mp4");
    
    if !input_path.exists() {
        eprintln!("Skipping test: {} not found", input_path.display());
        return;
    }

    fs::create_dir_all("test_videos/output").unwrap();

    // Test different size presets
    let sizes = vec!["1mb", "5mb", "10mb"];
    let expected_sizes = vec![1.1, 5.25, 10.5];

    for (size_preset, max_size) in sizes.iter().zip(expected_sizes.iter()) {
        let output_path = get_output_path(&format!("cli_test_720p_{}.mp4", size_preset));
        let _ = fs::remove_file(&output_path);

        let mut cmd = Command::cargo_bin("small-mp4").unwrap();
        cmd.arg("compress")
            .arg(&input_path)
            .arg("-o")
            .arg(&output_path)
            .arg("--size")
            .arg(size_preset)
            .arg("--force-software")
            .assert()
            .success();

        assert!(output_path.exists());
        
        let metadata = fs::metadata(&output_path).unwrap();
        let size_mb = metadata.len() as f32 / (1024.0 * 1024.0);
        assert!(size_mb <= *max_size, "Output size {} MB exceeds target for {}", size_mb, size_preset);
    }
}

#[test]
fn test_cli_compress_invalid_input() {
    let mut cmd = Command::cargo_bin("small-mp4").unwrap();
    cmd.arg("compress")
        .arg("nonexistent_file.mp4")
        .assert()
        .failure();
}

#[test]
fn test_cli_compress_invalid_size() {
    let input_path = get_test_video_path("test_720p_10s.mp4");
    
    if !input_path.exists() {
        eprintln!("Skipping test: {} not found", input_path.display());
        return;
    }

    let mut cmd = Command::cargo_bin("small-mp4").unwrap();
    cmd.arg("compress")
        .arg(&input_path)
        .arg("--size")
        .arg("invalid")
        .assert()
        .failure();
}