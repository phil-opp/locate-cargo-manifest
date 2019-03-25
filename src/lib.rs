//! Retrieves the Cargo manifest path by parsing the output of `cargo locate-project`.
//!
//! Example:
//! 
//! ```
//! use locate_cargo_manifest::locate_manifest;
//!
//! let manifest_path = locate_manifest().expect("failed to retrieve cargo manifest path");
//! assert!(manifest_path.ends_with("Cargo.toml"));
//! ```

#![warn(missing_docs)]

use std::{env, io, path::{PathBuf}, process, string};

/// Returns the Cargo manifest path of the surrounding crate.
///
/// The path is retrieved by parsing the output of `cargo locate-project`.
pub fn locate_manifest() -> Result<PathBuf, LocateManifestError> {
    let cargo = env::var("CARGO").unwrap_or("cargo".to_owned());
    let output = process::Command::new(cargo).arg("locate-project").output()?;
    if !output.status.success() {
        return Err(LocateManifestError::CargoExecution{ stderr: output.stderr});
    }

    let output = String::from_utf8(output.stdout)?;
    let parsed = json::parse(&output)?;
    let root = parsed["root"].as_str().ok_or(LocateManifestError::NoRoot)?;
    Ok(PathBuf::from(root))
}

/// Errors that can occur while retrieving the cargo manifest path.
#[derive(Debug)]
pub enum LocateManifestError {
    /// An I/O error that occurred while trying to execute `cargo locate-project`.
    Io(io::Error),
    /// The command `cargo locate-project` did not exit successfully.
    CargoExecution{
        /// The standard error output of `cargo locate-project`.
        stderr: Vec<u8>,
    },
    /// The output of `cargo locate-project` was not valid UTF-8.
    StringConversion(string::FromUtf8Error),
    /// An error occurred while parsing the output of `cargo locate-project` as JSON.
    ParseJson(json::Error),
    /// The JSON output of `cargo locate-project` did not contain the expected "root" string.
    NoRoot,
}

impl From<io::Error> for LocateManifestError {
    fn from(err: io::Error) -> Self {
        LocateManifestError::Io(err)
    }
}

impl From<string::FromUtf8Error> for LocateManifestError {
    fn from(err: string::FromUtf8Error) -> Self {
        LocateManifestError::StringConversion(err)
    }
}

impl From<json::Error> for LocateManifestError {
    fn from(err: json::Error) -> Self {
        LocateManifestError::ParseJson(err)
    }
}

#[test]
fn test_manifest_path() {
    use std::path::Path;

    let manifest_path = locate_manifest().expect("failed to retrieve cargo manifest path");
    let manual_path = Path::new(file!()).parent().unwrap().join("../Cargo.toml").canonicalize().unwrap();
    assert_eq!(manifest_path, manual_path);
}