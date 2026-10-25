// SPDX-License-Identifier: MIT
use super::*;
use std::path::{Path, PathBuf};
pub const REFERENCE_CORE: &str = "eaefdb4f61d01133557374585383e531438e0341d11579c11e8575283a5ccb9b";
pub const HEADLESS_CORE: &str = "554f6164155b673bd97be067d435f98bebe54c73dd0fa556e7480de9035e543f";
pub const CORE: &str = "b302cafae4f049e6ec61294538acdded43346302322bc092deb8dbe69008ead6";
pub const OPEN: &str = "c29e4f5edb36807ef497e0ad8a63a984bea26daa1c075285f496815501f079b8";
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
    pub(crate) window: bool,
}
impl Resources {
    /// The normal runtime accepts only the compiled supported open resource set.
    /// No proprietary/reference fallback, config override or firmware selector.
    pub fn open(root: &Path) -> io::Result<Self> {
        Self::validated(root, OPEN, false)
    }
    /// Exact accepted logging build, used only by the desktop qualification
    /// owner. The ordinary player cannot select this development configuration.
    #[cfg(not(target_os = "android"))]
    pub(crate) fn logging(root: &Path) -> io::Result<Self> {
        Self::validated(root, LOGGING_OPEN, true)
    }
    fn validated(root: &Path, open: &str, window: bool) -> io::Result<Self> {
        let root = root.canonicalize()?;
        let m: Manifest = read_json(&root.join("runtime.json"))?;
        if m.schema != 1 || m.core != CORE || m.open_system != open || m.state_abi != 1 {
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
        self.root.join("native/core")
    }
    pub fn image_tool(&self) -> PathBuf {
        self.root.join("native/qemu-img")
    }
}
