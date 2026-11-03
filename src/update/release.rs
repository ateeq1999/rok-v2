//! rok-v2's GitHub releases, as `rok update` reads them: which archive this
//! platform takes, which version is newer, the archive's checksum, and the
//! `rok` binary inside it.

use std::io::{Cursor, Read};

use sha2::{Digest, Sha256};

/// Where the latest release is described, unless `ROK_UPDATE_API` says
/// otherwise (the tests serve their own).
const LATEST_API: &str = "https://api.github.com/repos/ateeq1999/rok-v2/releases/latest";

/// Where release archives are downloaded from, unless `ROK_UPDATE_DOWNLOADS`
/// says otherwise.
const DOWNLOADS: &str = "https://github.com/ateeq1999/rok-v2/releases/download";

/// The largest archive `rok update` reads (the release archives are a few
/// megabytes).
const LIMIT: u64 = 256 * 1024 * 1024;

/// A version as releases name it: `X.Y.Z`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Version(pub(crate) u64, pub(crate) u64, pub(crate) u64);

impl Version {
    /// `0.7.1` or `v0.7.1`; a pre-release or build suffix is not a release
    /// `rok update` installs.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let mut parts = text.trim().trim_start_matches('v').split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        parts.next().is_none().then_some(Self(major, minor, patch))
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

/// This binary's version.
pub(crate) fn current() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("the crate version is X.Y.Z")
}

/// The release archive for `target` (a target triple): Linux builds take
/// the static musl archive, whichever libc they were built for.
pub(crate) fn asset_name(version: Version, target: &str) -> Option<String> {
    let published = match target {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => "aarch64-unknown-linux-musl",
        "x86_64-apple-darwin"
        | "aarch64-apple-darwin"
        | "x86_64-pc-windows-msvc"
        | "aarch64-pc-windows-msvc" => target,
        _ => return None,
    };
    let extension = if published.contains("windows") {
        "zip"
    } else {
        "tar.gz"
    };
    Some(format!("rok-{version}-{published}.{extension}"))
}

fn get(url: &str) -> Result<ureq::http::Response<ureq::Body>, String> {
    ureq::get(url)
        .header(
            "User-Agent",
            concat!("rok/", env!("CARGO_PKG_VERSION"), " (rok update)"),
        )
        .call()
        .map_err(|err| format!("can't fetch {url}: {err}"))
}

/// The latest release's version.
pub(crate) fn latest() -> Result<Version, String> {
    let url = std::env::var("ROK_UPDATE_API").unwrap_or_else(|_| LATEST_API.to_owned());
    let text = get(&url)?
        .body_mut()
        .read_to_string()
        .map_err(|err| format!("can't read {url}: {err}"))?;
    let release: serde_json::Value =
        serde_json::from_str(&text).map_err(|err| format!("{url} is not a release: {err}"))?;
    let tag = release["tag_name"]
        .as_str()
        .ok_or_else(|| format!("{url} names no tag_name"))?;
    Version::parse(tag).ok_or_else(|| format!("the latest release's tag `{tag}` is not vX.Y.Z"))
}

/// The bytes at the release asset `name` of `version`.
pub(crate) fn download(version: Version, name: &str) -> Result<Vec<u8>, String> {
    let base = std::env::var("ROK_UPDATE_DOWNLOADS").unwrap_or_else(|_| DOWNLOADS.to_owned());
    let url = format!("{base}/v{version}/{name}");
    get(&url)?
        .body_mut()
        .with_config()
        .limit(LIMIT)
        .read_to_vec()
        .map_err(|err| format!("can't read {url}: {err}"))
}

/// Checks `archive` against its `.sha256` file (`<hex>  <name>`).
pub(crate) fn verify(archive: &[u8], checksum_file: &str, name: &str) -> Result<(), String> {
    let expected = checksum_file
        .split_whitespace()
        .next()
        .ok_or_else(|| format!("{name}.sha256 is empty"))?
        .to_ascii_lowercase();
    let actual: String = Sha256::digest(archive)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{name}'s SHA-256 is {actual}, but its checksum file says {expected}; nothing was installed"
        ))
    }
}

/// The `rok` binary inside a release archive (`<dir>/rok` in a `.tar.gz`,
/// `<dir>/rok.exe` in a `.zip`).
pub(crate) fn extract(archive: &[u8], name: &str) -> Result<Vec<u8>, String> {
    let wanted = |path: &str| {
        let file = path.rsplit(['/', '\\']).next().unwrap_or(path);
        file == "rok" || file == "rok.exe"
    };
    let mut binary = Vec::new();
    if name.ends_with(".zip") {
        let mut zip = zip::ZipArchive::new(Cursor::new(archive))
            .map_err(|err| format!("{name} is not a zip archive: {err}"))?;
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).map_err(|err| format!("{name}: {err}"))?;
            let path = file
                .name()
                .map_err(|err| format!("{name}: {err}"))?
                .into_owned();
            if file.is_file() && wanted(&path) {
                file.read_to_end(&mut binary)
                    .map_err(|err| format!("{name}: {err}"))?;
                return Ok(binary);
            }
        }
    } else {
        let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(archive));
        for entry in tar.entries().map_err(|err| format!("{name}: {err}"))? {
            let mut entry = entry.map_err(|err| format!("{name}: {err}"))?;
            let path = entry.path().map_err(|err| format!("{name}: {err}"))?;
            if entry.header().entry_type().is_file() && wanted(&path.to_string_lossy()) {
                entry
                    .read_to_end(&mut binary)
                    .map_err(|err| format!("{name}: {err}"))?;
                return Ok(binary);
            }
        }
    }
    Err(format!("{name} holds no rok binary"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_parse_and_order() {
        assert_eq!(Version::parse("v0.7.1"), Some(Version(0, 7, 1)));
        assert_eq!(Version::parse("1.2.3"), Some(Version(1, 2, 3)));
        assert_eq!(Version::parse("0.8.0-rc.1"), None);
        assert_eq!(Version::parse("1.2"), None);
        assert!(Version(0, 10, 0) > Version(0, 9, 9));
        assert_eq!(Version(0, 7, 1).to_string(), "0.7.1");
    }

    #[test]
    fn each_platform_takes_its_archive() {
        let v = Version(0, 8, 0);
        assert_eq!(
            asset_name(v, "x86_64-unknown-linux-gnu").as_deref(),
            Some("rok-0.8.0-x86_64-unknown-linux-musl.tar.gz")
        );
        assert_eq!(
            asset_name(v, "aarch64-apple-darwin").as_deref(),
            Some("rok-0.8.0-aarch64-apple-darwin.tar.gz")
        );
        assert_eq!(
            asset_name(v, "x86_64-pc-windows-msvc").as_deref(),
            Some("rok-0.8.0-x86_64-pc-windows-msvc.zip")
        );
        assert_eq!(asset_name(v, "riscv64gc-unknown-linux-gnu"), None);
    }

    #[test]
    fn a_checksum_mismatch_is_refused() {
        let archive = b"archive bytes";
        let good: String = Sha256::digest(archive)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert!(verify(archive, &format!("{good}  a.tar.gz\n"), "a.tar.gz").is_ok());
        let err = verify(
            archive,
            &format!("{}  a.tar.gz", "0".repeat(64)),
            "a.tar.gz",
        )
        .expect_err("a wrong checksum");
        assert!(err.contains("nothing was installed"), "{err}");
    }

    #[test]
    fn the_binary_comes_out_of_a_tar_gz() {
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        {
            let mut tar = tar::Builder::new(&mut gz);
            for (path, data) in [
                ("rok-0.8.0-x/README.md", &b"readme"[..]),
                ("rok-0.8.0-x/rok", b"the binary"),
            ] {
                let mut header = tar::Header::new_gnu();
                header.set_size(data.len() as u64);
                header.set_mode(0o755);
                header.set_cksum();
                tar.append_data(&mut header, path, data).expect("append");
            }
            tar.finish().expect("finish");
        }
        let archive = gz.finish().expect("gzip");
        assert_eq!(
            extract(&archive, "rok-0.8.0-x.tar.gz").expect("extract"),
            b"the binary"
        );
        assert!(extract(b"not an archive", "rok.tar.gz").is_err());
    }
}
