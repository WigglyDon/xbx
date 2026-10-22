# Source map and observation limits

Observed during seed preparation on 2026-10-21. Recheck changing claims at actual
qualification/release. Sources explain candidate behavior, not xbx results. A
source author saying "clean-room" or "legal" is not independent certification.

| ID | Primary source | Use in this seed |
| --- | --- | --- |
| S1 | [xemu source](https://github.com/xemu-project/xemu), observed `03b988c9d76553b33ed712ba00822ffdf49f0e7d` | Upstream reference lineage; source/build audit needed |
| S2 | [X1 BOX author](https://www.izzy2lost.com/x1box), [pinned source README](https://github.com/izzy2lost/xemu/blob/4967efd3ee594cc236869a45ee4cb396e25af1ac/README.md) | Author-reported Android ARM64/Vulkan port and requirements |
| S3 | [xemu snapshots](https://xemu.app/docs/snapshots/) | HDD inclusion and old-snapshot rollback warning |
| S4 | [xemu required files](https://xemu.app/docs/required-files/) | Conventional boot/flash/disk inputs; independently distributable disk route |
| S5 | [xemu disc images](https://xemu.app/docs/disc-images/) | XISO distinction and supported content handling |
| S6 | [xemu download](https://xemu.app/docs/download/), [about](https://xemu.app/docs/about/) | Published desktop/aarch64 routes; not Fold performance evidence |
| S7 | [Cxbx-Reloaded](https://github.com/Cxbx-Reloaded/Cxbx-Reloaded) | Windows/Wine/Direct3D host constraints, not a ready ARM Android donor |
| S8 | [Roswell](https://github.com/XboxDev/roswell/tree/1569e2e89fb47cc72b9c704a8884f98200432bd4) | Work-in-progress open kernel; README's no-bootrom/128 MB route and license summary |
| S9 | [OXOS](https://github.com/kooscode/oxos/tree/c0c887a1263ebf89ba765f34a88ca8f132901e25) | Author claims of open BIOS/kernel, selected games and separate proprietary comparison route; audit required |
| S10 | [Cromwell](https://github.com/XboxDev/cromwell) | Not a retail Xbox-game BIOS replacement on its own |
| S11 | [xemu LICENSE](https://github.com/xemu-project/xemu/blob/03b988c9d76553b33ed712ba00822ffdf49f0e7d/LICENSE) | Review actual GPL/component terms before incorporation |
| S12 | [GNU license FAQ: translation](https://www.gnu.org/licenses/gpl-faq.en.html#TranslateCode) | Translation does not erase inherited copyright/license |
| S13 | [Android 16 KB pages](https://developer.android.com/guide/practices/page-sizes) | Native build and actual runtime qualification are different |
| S14 | [Apple App Review Guidelines](https://developer.apple.com/app-store/review/guidelines/) | Emulator category and remaining review requirements |
| S15 | [Apple runtime security](https://support.apple.com/guide/security/security-of-runtime-process-sec15bfe098e/web) | Stock-iOS executable-memory restrictions |
| S16 | [GitHub CLI create](https://cli.github.com/manual/gh_repo_create), [edit](https://cli.github.com/manual/gh_repo_edit), [API](https://cli.github.com/manual/gh_api) | Local public-repo bootstrap, not a claim that creation ran here |
| S17 | [Codex CLI reference](https://developers.openai.com/codex/cli/reference/) | Normal interactive opening prompt; no approval-bypass flags |
| S18 | [PSX instructions](https://github.com/WigglyDon/psx/blob/1e4ba86fa0a5fc1470837b6bcc06bd20b6642c11/AGENTS.md), [storage](https://github.com/WigglyDon/psx/blob/1e4ba86fa0a5fc1470837b6bcc06bd20b6642c11/docs/storage.md) | Pinned sister-product decisions and resource lessons |

Candidate commits were observed through GitHub reads. The entire dependency
closure, provenance, binary correspondence, APIs, performance and compatibility
were **not** audited by this seed. No third-party implementation or media was
copied. The new Master must evaluate the exact source it actually uses.
