//! Original finite XBE fixture layout, derived from public format/interface facts.
//! No SDK, game bytes, or copied loader/wrapper implementation.
use crate::{kernel, run};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn put(image: &mut [u8], at: usize, value: u32) {
    image[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn digest(bytes: &[u8]) -> Result<[u8; 20], Box<dyn std::error::Error>> {
    let mut command = Command::new("sha1sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut input = command.stdin.take().ok_or("sha1 stdin missing")?;
    input.write_all(&(bytes.len() as u32).to_le_bytes())?;
    input.write_all(bytes)?;
    drop(input);
    let out = command.wait_with_output()?;
    if !out.status.success() {
        return Err("fixture digest failed".into());
    }
    let text = String::from_utf8(out.stdout)?;
    let mut result = [0; 20];
    for (i, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16)?;
    }
    Ok(result)
}

pub fn build(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let _lease = crate::capacity::writer(root, 1)?;
    let source = root.join(format!(".local/src/oxos-{}", kernel::OXOS));
    kernel::verify(
        &source,
        kernel::OXOS,
        Some(&fs::read(root.join("patches/oxos-launch-trace.patch"))?),
    )?;
    let build = root.join(".local/build-loader-fixture-v1");
    fs::create_dir(&build)?;
    let llvm = PathBuf::from(std::env::var_os("XBX_NDK").ok_or("set XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    run(Command::new(llvm.join("clang"))
        .args(["--target=i386-unknown-none-elf", "-march=pentium3", "-c"])
        .arg(root.join("native/fixtures/loader-complete.S"))
        .arg("-o")
        .arg(build.join("title.o")))?;
    run(Command::new(llvm.join("llvm-objcopy"))
        .args(["-O", "binary", "-j", ".text"])
        .arg(build.join("title.o"))
        .arg(build.join("text.raw")))?;
    let code = fs::read(build.join("text.raw"))?;
    if code.is_empty() || code.len() > 0x800 {
        return Err("fixture code extent invalid".into());
    }
    let mut image = vec![0u8; 0x440];
    image[..4].copy_from_slice(b"XBEH");
    for (at, value) in [
        (0x104, 0x10000),
        (0x108, 0x440),
        (0x10c, 0x3000),
        (0x110, 0x178),
        (0x118, 0x10178),
        (0x11c, 3),
        (0x120, 0x10348),
        (0x128, 0x11000 ^ 0xa8fc57ab),
        (0x130, 0x10000),
        (0x134, 0x100000),
        (0x138, 0x1000),
        (0x158, 0x10430 ^ 0x5b6d40b6),
        (0x178, 0x1d0),
        (0x180, 0x58425803),
        (0x214, 0x00ffffff),
        (0x218, 0x7fffffff),
    ] {
        put(&mut image, at, value);
    }
    for (i, c) in "xbx exact loader".encode_utf16().enumerate() {
        image[0x184 + i * 2..0x186 + i * 2].copy_from_slice(&c.to_le_bytes());
    }
    image[0x410..0x422].copy_from_slice(b".text\0.init\0.tail\0");
    let mut initialized = Vec::new();
    for _ in 0..512 {
        initialized.extend_from_slice(&0x11223344u32.to_le_bytes());
    }
    initialized.resize(0x820, 0); // guest results; following 0x7e0 bytes are BSS
    let mut tail = Vec::new();
    for _ in 0..64 {
        tail.extend_from_slice(&0x55667788u32.to_le_bytes());
    }
    let sections = [
        (
            6u32, 0x11000u32, 0x800u32, 0x10410u32, 0x10400u32, 0x10402u32, code,
        ),
        (3, 0x11800, 0x1000, 0x10416, 0x10402, 0x10404, initialized),
        (3, 0x12800, 0x800, 0x1041c, 0x10404, 0x10406, tail),
    ];
    for (i, (flags, va, virtual_size, name, head, tail, data)) in sections.iter().enumerate() {
        let at = 0x348 + i * 0x38;
        let raw = image.len() as u32;
        for (offset, value) in [
            (0, *flags),
            (4, *va),
            (8, *virtual_size),
            (12, raw),
            (16, data.len() as u32),
            (20, *name),
            (28, *head),
            (32, *tail),
        ] {
            put(&mut image, at + offset, value);
        }
        image[at + 36..at + 56].copy_from_slice(&digest(data)?);
        image.extend_from_slice(data);
        image.resize((image.len() + 15) & !15, 0);
    }
    if image.len() >= 0x1000 {
        return Err("valid fixture must exercise a sufficient short initial probe".into());
    }
    let mut truncated = image.clone();
    truncated.truncate(image.len() - 64);
    let mut header_short = image[..0x440].to_vec();
    header_short.resize(0x1400, 0);
    put(&mut header_short, 0x108, 0x1800);
    for i in 0..3 {
        put(
            &mut header_short,
            0x348 + i * 0x38 + 12,
            0x1800 + (i as u32) * 0x1000,
        );
    }
    let mut permissive = image.clone();
    permissive[0x348 + 36] ^= 1; // one genuine mismatch, retained permissive policy
    permissive[0x380 + 36..0x380 + 56].fill(0); // one absent/unusable digest
    for (name, bytes) in [
        ("valid", image),
        ("truncated", truncated),
        ("header-short", header_short),
        ("permissive", permissive),
    ] {
        fs::write(build.join(format!("{name}.xbe")), &bytes)?;
        // The retained writer records len(bytes) as the directory length;
        // sector padding is never presented as valid file content.
        run(Command::new("python3").args(["-I", "-c", "import sys;sys.path.insert(0,sys.argv[1]);import mkxiso;mkxiso.write_image(sys.argv[2],{'default.xbe':open(sys.argv[3],'rb').read()})"])
            .arg(source.join("tests")).arg(build.join(format!("{name}.iso"))).arg(build.join(format!("{name}.xbe"))))?;
    }
    fs::write(
        build.join("title.map"),
        " 0001:00000000 _XkTitleResults 0000000000012000 fixture\n",
    )?;
    println!("Original loader fixtures retained at {}", build.display());
    Ok(())
}
