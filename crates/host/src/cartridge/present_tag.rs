// SPDX-License-Identifier: MIT
//! Bounded producer marker after an actual captured old/new picture recurrence.
use super::{self as c, present_semaphore};
use serde_json::json;
use std::{fs, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const UNIT: &str = "hw/xbox/nv2a/pgraph/vk/display.c";
const PATCH: &str = "patches/xemu-present-tag.patch";
const PARENT: &str = "c5dff2b5205a467fcc85f46e7f3458b748ecd89a403df5eea649ccc7080e4652";

fn declaration<'a>(text: &'a str, start: &str, end: &str) -> Result<&'a str, Error> {
    let a = text
        .find(start)
        .ok_or("retained shader declaration missing")?;
    let b = text[a..]
        .find(end)
        .ok_or("retained shader boundary missing")?;
    Ok(&text[a..a + b])
}
fn tag_offset(bytes: &[u8]) -> Result<u32, Error> {
    if bytes.len() < 20 || !bytes.len().is_multiple_of(4) {
        return Err("invalid generated SPIR-V size".into());
    }
    let words: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
        .collect();
    if words[0] != 0x07230203 {
        return Err("invalid generated SPIR-V header".into());
    }
    let mut member = None;
    let mut offsets = Vec::new();
    let mut at = 5;
    while at < words.len() {
        let count = (words[at] >> 16) as usize;
        if count == 0 || count > words.len() - at {
            return Err("invalid generated SPIR-V instruction".into());
        }
        let w = &words[at..at + count];
        match w[0] & 0xffff {
            6 if count >= 4 => {
                let name: Vec<u8> = w[3..].iter().flat_map(|v| v.to_le_bytes()).collect();
                if name.split(|b| *b == 0).next() == Some(b"present_tag")
                    && member.replace((w[1], w[2])).is_some()
                {
                    return Err("duplicate generated marker member".into());
                }
            }
            72 if count == 5 && w[3] == 35 => offsets.push(((w[1], w[2]), w[4])),
            _ => {}
        }
        at += count;
    }
    let member = member.ok_or("generated marker member missing")?;
    let mut matching = offsets.iter().filter(|(key, _)| *key == member);
    let offset = matching.next().ok_or("generated marker offset missing")?.1;
    if matching.next().is_some() || offset != 112 || offset + 4 > 128 {
        return Err("marker exceeds expected portable push-constant layout".into());
    }
    Ok(offset)
}
pub(super) fn contract(repo: &Path, root: &Path) -> Result<(), Error> {
    let parent = root.join("android-demo-v1/android-present-semaphore-v1");
    if c::file_digest(&parent.join("libxemu.so"))? != PARENT {
        return Err("qualified marker parent differs".into());
    }
    let out = root.join("native-present-tag-contract-v1");
    c::mkdir(&out)?;
    let source = out.join("source");
    fs::create_dir_all(source.join("hw/xbox/nv2a/pgraph/vk"))?;
    let before = fs::read_to_string(parent.join("source").join(UNIT))?;
    c::write_new(&source.join(UNIT), before.as_bytes())?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .env("GIT_CEILING_DIRECTORIES", &out)
            .arg("apply")
            .arg(repo.join(PATCH)),
    )?;
    let after = fs::read_to_string(source.join(UNIT))?;
    if present_semaphore::subject(&before)? != present_semaphore::subject(&after)? {
        return Err("marker changes the retained submission/presentation block".into());
    }
    let shader = declaration(
        &after,
        "static const char *display_frag_glsl =",
        "\nstatic void create_descriptor_pool",
    )?;
    let limits =
        fs::read_to_string(repo.join(".local/src/xemu-android/hw/xbox/nv2a/pgraph/vk/glsl.c"))?;
    let resources = declaration(
        &limits,
        "static const glslang_resource_t\n",
        "\nvoid pgraph_vk_init_glsl_compiler",
    )?;
    let include = out.join("retained-shader.inc");
    c::write_new(
        &include,
        format!("/* SPDX-License-Identifier: LGPL-2.0-or-later */\n{resources}\n{shader}\n")
            .as_bytes(),
    )?;
    let headers = repo.join(".local/build-android/_deps/glslang-src");
    let libraries = repo.join(".local/build-desktop/subprojects/glslang");
    let object = out.join("fixture.o");
    c::store::checked(
        Command::new("cc")
            .args(["-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", "-I"])
            .arg(&headers)
            .arg(format!("-DSUBJECT=\"{}\"", include.display()))
            .arg("-c")
            .arg(repo.join("native/fixtures/present-tag-shader.c"))
            .arg("-o")
            .arg(&object),
    )?;
    let binary = out.join("shader-compile");
    let mut link = Command::new("c++");
    link.arg(&object).arg("-Wl,--start-group");
    let mut library_hashes = Vec::new();
    for name in [
        "libglslang.a",
        "libMachineIndependent.a",
        "libGenericCodeGen.a",
        "libSPIRV.a",
    ] {
        let path = libraries.join(name);
        library_hashes.push(json!({"name":name,"sha256":c::file_digest(&path)?}));
        link.arg(path);
    }
    link.args(["-Wl,--end-group", "-pthread", "-o"])
        .arg(&binary);
    c::store::checked(&mut link)?;
    let spv = out.join("display.spv");
    c::store::checked(Command::new(&binary).arg(&spv))?;
    let offset = tag_offset(&fs::read(&spv)?)?;
    c::write_new(
        &out.join("result.json"),
        &c::json(
            &json!({"passed":true,"parent_core":PARENT,"patch":c::file_digest(&repo.join(PATCH))?,"fixture":c::file_digest(&repo.join("native/fixtures/present-tag-shader.c"))?,"before_source":c::digest(before.as_bytes()),"after_source":c::digest(after.as_bytes()),"submission_presentation_block_bytes_unchanged":true,"shader_SPIRV":c::file_digest(&spv)?,"shader_marker_offset":offset,"shader_marker_end":offset+4,"portable_push_constant_limit":128,"compiler_header":c::file_digest(&headers.join("glslang/Include/glslang_c_interface.h"))?,"retained_host_compiler_libraries":library_hashes,"marker":"28 binary cells:16 sequence,3 acquired-slot,overlay enabled/use0/use1,3 ephemeral binding-token,3 fixed sync bits; black/white border","guest_address_code_stack_or_raw_native_handle_collected":false,"actual_Vulkan_GPU_execution":false,"cause_or_performance_fix_qualified":false}),
        )?,
    )?;
    Ok(())
}
