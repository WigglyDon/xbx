//! The first xbx diagnostic host. No owner media or writable profiles are fixtures.
#[cfg(not(target_os = "android"))]
pub mod api_journal;
pub mod branch;
pub mod callsite;
#[cfg(not(target_os = "android"))]
pub mod desktop_title;
pub mod disc;
pub mod disk;
pub mod input;
#[cfg(not(target_os = "android"))]
pub mod observer;
#[cfg(not(target_os = "android"))]
pub mod probe;
pub mod runtime_summary;
pub mod session;
pub mod startup;
pub mod storage;

/// Fixed private-profile paths; no arbitrary donor options or automatic snapshot restore.
pub fn profile_config(root: &std::path::Path) -> std::io::Result<String> {
    let root = root.canonicalize()?;
    let quoted = |name: &str| -> std::io::Result<String> {
        let path = root.join(name);
        let text = path
            .to_str()
            .ok_or_else(|| std::io::Error::other("non-UTF8 profile path"))?;
        if text.chars().any(char::is_control) {
            return Err(std::io::Error::other("control character in profile path"));
        }
        Ok(format!(
            "\"{}\"",
            text.replace('\\', "\\\\").replace('"', "\\\"")
        ))
    };
    Ok(format!(
        "[general]\nshow_welcome = false\nskip_boot_anim = false\n[general.updates]\ncheck = false\n[display]\nrenderer = \"vulkan\"\n[sys]\nmem_limit = \"64\"\n[sys.files]\nbootrom_path = \"\"\nflashrom_path = {}\nhdd_path = {}\ndvd_path = \"/dev/fdset/1\"\neeprom_path = {}\n",
        quoted("flash.bin")?,
        quoted("hdd.qcow2")?,
        quoted("eeprom.bin")?
    ))
}

#[cfg(target_os = "android")]
mod android;
