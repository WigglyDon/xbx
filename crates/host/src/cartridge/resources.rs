// SPDX-License-Identifier: MIT
use super::*;
use std::path::{Path, PathBuf};
pub const REFERENCE_CORE: &str = "eaefdb4f61d01133557374585383e531438e0341d11579c11e8575283a5ccb9b";
pub const HEADLESS_CORE: &str = "554f6164155b673bd97be067d435f98bebe54c73dd0fa556e7480de9035e543f";
pub const CORE: &str = "b302cafae4f049e6ec61294538acdded43346302322bc092deb8dbe69008ead6";
pub const X87_RESTART_CORE: &str =
    "b8d9ce80080700c6a00a2678ca056452e905d80d901936047f581a76e9afb318";
pub const OPEN: &str = "c29e4f5edb36807ef497e0ad8a63a984bea26daa1c075285f496815501f079b8";
pub const NPX_LOGGING_OPEN: &str =
    "8477c0477e1e9bac6ba9446dcd020293e46c5cb1fe5320449ae10f4c486dc11b";
pub const FLUSH_LOGGING_OPEN: &str =
    "c94bfdb0aefa127e62d91d0377b55f75cf0023e485b5be3931727490cd595b0a";
pub const LOGGING_OPEN: &str = "8f1d5ac9ec1b8d97eabf13fdca53f0f995f19b56951e1ddf8dca370cf71c070e";
pub const ORIGINAL_FIXTURES: [&str; 2] = [
    "317e10827bc88ae6e5dcb768c242867d7bc7d6dde963ae9e567a9340179c591a",
    "85a173c73aa642fd82f64146149c6a37c38720ac8a020eb03d0c4f34de6c3255",
];
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub core: String,
    pub open_system: String,
    pub disk_tool: String,
    pub slirp: String,
    pub state_abi: u32,
}
pub struct Resources {
    pub root: PathBuf,
    pub identity: String,
    pub manifest: Manifest,
    #[cfg(not(target_os = "android"))]
    pub(crate) window: bool,
    #[cfg(target_os = "android")]
    pub(crate) native_dir: PathBuf,
}
impl Resources {
    /// The normal runtime accepts only the compiled supported open resource set.
    /// No proprietary/reference fallback, config override or firmware selector.
    #[cfg(not(target_os = "android"))]
    pub fn open(root: &Path) -> io::Result<Self> {
        Self::validated(root, OPEN, false)
    }
    /// Exact accepted logging build, used only by the desktop qualification
    /// owner. The ordinary player cannot select this development configuration.
    #[cfg(not(target_os = "android"))]
    pub(crate) fn logging(root: &Path) -> io::Result<Self> {
        Self::validated(root, LOGGING_OPEN, true)
    }
    /// Exact independently qualified functional-port candidate; developer only.
    #[cfg(not(target_os = "android"))]
    pub(crate) fn npx_logging(root: &Path) -> io::Result<Self> {
        Self::validated(root, NPX_LOGGING_OPEN, true)
    }
    /// Pinned original-qualification candidate, unreachable from ordinary open.
    #[cfg(not(target_os = "android"))]
    pub(crate) fn x87_logging(root: &Path) -> io::Result<Self> {
        Self::validated_core(root, X87_RESTART_CORE, NPX_LOGGING_OPEN, true)
    }
    /// Exact queued-flush development candidate; not a product selector.
    #[cfg(not(target_os = "android"))]
    pub(crate) fn flush_logging(root: &Path) -> io::Result<Self> {
        Self::validated_core(root, X87_RESTART_CORE, FLUSH_LOGGING_OPEN, true)
    }
    #[cfg(not(target_os = "android"))]
    fn validated(root: &Path, open: &str, window: bool) -> io::Result<Self> {
        Self::validated_core(root, CORE, open, window)
    }
    #[cfg(not(target_os = "android"))]
    fn validated_core(root: &Path, core: &str, open: &str, window: bool) -> io::Result<Self> {
        let root = root.canonicalize()?;
        let m: Manifest = read_json(&root.join("runtime.json"))?;
        if m.schema != 1 || m.core != core || m.open_system != open || m.state_abi != 1 {
            return Err(io::Error::other(
                "unsupported integrated runtime; session and saves retained",
            ));
        }
        for (p, h) in [
            ("native/core", &m.core),
            ("open.bin", &m.open_system),
            ("native/qemu-img", &m.disk_tool),
            ("native/subprojects/slirp/libslirp.so.0", &m.slirp),
        ] {
            if file_digest(&root.join(p))? != *h {
                return Err(io::Error::other("packaged runtime damaged"));
            }
        }
        Ok(Self {
            // Auxiliary packaging/build tools and frontend changes are not
            // machine-state compatibility identities.
            identity: digest(
                format!(
                    "xbx-state:{}:{}:{}:64:offline",
                    m.state_abi, m.core, m.open_system
                )
                .as_bytes(),
            ),
            root,
            manifest: m,
            window,
        })
    }
    pub fn core(&self) -> PathBuf {
        #[cfg(target_os = "android")]
        {
            self.native_dir.join("libxemu.so")
        }
        #[cfg(not(target_os = "android"))]
        {
            self.root.join("native/core")
        }
    }
    pub fn image_tool(&self) -> PathBuf {
        #[cfg(target_os = "android")]
        {
            self.native_dir.join("libxbx-disk.so")
        }
        #[cfg(not(target_os = "android"))]
        {
            self.root.join("native/qemu-img")
        }
    }
}

#[cfg(target_os = "android")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AndroidPackage {
    schema: u32,
    core: String,
    open_system: String,
    image_helper: String,
    empty_disk: String,
    state_abi: u32,
    bridge_abi: u32,
}
#[cfg(target_os = "android")]
impl Resources {
    pub fn open(root: &Path) -> io::Result<Self> {
        unsafe extern "C" {
            fn xbx_platform_native_dir() -> *const std::ffi::c_char;
        }
        let pointer = unsafe { xbx_platform_native_dir() };
        if pointer.is_null() {
            return Err(io::Error::other("Android platform resources are not ready"));
        }
        let native = unsafe { std::ffi::CStr::from_ptr(pointer) }
            .to_str()
            .map_err(io::Error::other)?;
        Self::android(root, Path::new(native))
    }
    pub(crate) fn android(root: &Path, native_dir: &Path) -> io::Result<Self> {
        let root = root.canonicalize()?;
        let native_dir = native_dir.canonicalize()?;
        let m: AndroidPackage = read_json(&root.join("runtime.json"))?;
        let expected_core = option_env!("XBX_ANDROID_CORE_SHA256")
            .ok_or_else(|| io::Error::other("Android native package identity missing"))?;
        let expected_helper = option_env!("XBX_ANDROID_DISK_SHA256")
            .ok_or_else(|| io::Error::other("Android disk-helper identity missing"))?;
        let expected_seed = option_env!("XBX_ANDROID_SEED_SHA256")
            .ok_or_else(|| io::Error::other("generated Android seed identity missing"))?;
        if m.schema != 1
            || m.state_abi != 1
            || m.bridge_abi != 4
            || m.open_system != OPEN
            || m.core != expected_core
            || m.image_helper != expected_helper
            || m.empty_disk != expected_seed
        {
            return Err(io::Error::other(
                "unsupported integrated Android resources; preserve sessions and saves",
            ));
        }
        for (path, hash) in [
            (native_dir.join("libxemu.so"), &m.core),
            (native_dir.join("libxbx-disk.so"), &m.image_helper),
            (root.join("open.bin"), &m.open_system),
            (root.join("empty.qcow2"), &m.empty_disk),
        ] {
            if file_digest(&path)? != *hash {
                return Err(io::Error::other("integrated Android resource damaged"));
            }
        }
        Ok(Self {
            identity: digest(
                format!(
                    "xbx-state:{}:{}:{}:64:offline",
                    m.state_abi, m.core, m.open_system
                )
                .as_bytes(),
            ),
            root,
            native_dir,
            manifest: Manifest {
                schema: 1,
                core: m.core,
                open_system: m.open_system,
                disk_tool: m.image_helper,
                slirp: String::new(),
                state_abi: 1,
            },
        })
    }
}
