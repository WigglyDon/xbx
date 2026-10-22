use crate::run;
use std::fs;
use std::path::Path;
use std::process::Command;

pub const PIN: &str = "4967efd3ee594cc236869a45ee4cb396e25af1ac";
const URL: &str = "https://github.com/izzy2lost/xemu.git";

fn git(source: &Path, args: &[&str]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let result = Command::new("git")
        .arg("-C")
        .arg(source)
        .args(args)
        .output()?;
    if !result.status.success() {
        return Err("source identity command failed".into());
    }
    Ok(result.stdout)
}

pub fn verify(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = root.join(".local/src/xemu-android");
    if String::from_utf8(git(&source, &["rev-parse", "HEAD"])?)?.trim() != PIN {
        return Err("candidate HEAD differs from the selected pin; existing work preserved".into());
    }
    if String::from_utf8(git(&source, &["config", "--get", "remote.origin.url"])?)?.trim() != URL {
        return Err("unexpected source remote; existing work preserved".into());
    }
    let actual = git(&source, &["diff", "--binary", "HEAD"])?;
    if actual != fs::read(root.join("patches/xemu-android.patch"))? {
        return Err(
            "source changes differ from the reviewed patch; existing work preserved".into(),
        );
    }
    if String::from_utf8(git(
        &source,
        &["status", "--porcelain", "--untracked-files=all"],
    )?)?
    .lines()
    .any(|line| line.starts_with("??"))
    {
        return Err("unrecorded source files need review before building".into());
    }
    Ok(())
}

pub fn prepare(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let _lease = crate::capacity::writer(root, 1)?;
    let source = root.join(".local/src/xemu-android");
    if !source.exists() {
        fs::create_dir_all(source.parent().unwrap())?;
        run(Command::new("git")
            .args(["clone", "--depth", "1", "--no-checkout", URL])
            .arg(&source))?;
        run(Command::new("git")
            .arg("-C")
            .arg(&source)
            .args(["fetch", "--depth", "1", "origin", PIN]))?;
        run(Command::new("git")
            .arg("-C")
            .arg(&source)
            .args(["checkout", "--detach", PIN]))?;
    }
    if verify(root).is_ok() {
        return Ok(());
    }
    if String::from_utf8(git(&source, &["config", "--get", "remote.origin.url"])?)?.trim() != URL {
        return Err("unexpected source remote; existing work preserved".into());
    }
    if String::from_utf8(git(&source, &["rev-parse", "HEAD"])?)?.trim() != PIN
        || !git(&source, &["status", "--porcelain", "--untracked-files=all"])?.is_empty()
    {
        return Err(
            "existing source differs or has unfinished changes; inspect it instead of resetting"
                .into(),
        );
    }
    let patch = root.join("patches/xemu-android.patch");
    run(Command::new("git")
        .arg("-C")
        .arg(&source)
        .args(["apply", "--check"])
        .arg(&patch))?;
    run(Command::new("git")
        .arg("-C")
        .arg(&source)
        .arg("apply")
        .arg(patch))?;
    verify(root)
}
