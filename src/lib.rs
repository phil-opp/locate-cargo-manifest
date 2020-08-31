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

use std::{convert, env, fmt, io, path::PathBuf, process, string};

/// Returns the Cargo manifest path of the surrounding crate.
///
/// The path is retrieved by parsing the output of `cargo locate-project`.
pub fn locate_manifest() -> Result<PathBuf, LocateManifestError> {
    let cargo = env::var("CARGO").unwrap_or("cargo".to_owned());
    let output = process::Command::new(cargo)
        .arg("locate-project")
        .output()?;
    if !output.status.success() {
        return Err(LocateManifestError::CargoExecution {
            stderr: output.stderr,
        });
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
    CargoExecution {
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

impl fmt::Display for LocateManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocateManifestError::Io(err) => {
                write!(f, "An I/O error occurred while trying to execute `cargo locate-project`: {}", err)
            }
            LocateManifestError::CargoExecution { stderr } => {
                write!(f, "The command `cargo locate-project` did not exit successfully.\n\
                Stderr: {}", String::from_utf8_lossy(stderr))
            }
            LocateManifestError::StringConversion(err) => {
                write!(f, "The output of `cargo locate-project` was not valid UTF-8: {}", err)
            }
            LocateManifestError::ParseJson(err) => {
                write!(f, "The output of `cargo locate-project` was not valid JSON: {}", err)
            }
            LocateManifestError::NoRoot => {
                write!(f, "The JSON output of `cargo locate-project` did not contain the expected \"root\" string.")
            }
        }
    }
}

impl std::error::Error for LocateManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LocateManifestError::Io(err) => Some(err),
            LocateManifestError::CargoExecution { stderr: _ } => None,
            LocateManifestError::StringConversion(err) => Some(err),
            LocateManifestError::ParseJson(err) => Some(err),
            LocateManifestError::NoRoot => None,
        }
    }
}

impl convert::From<io::Error> for LocateManifestError {
    fn from(source: io::Error) -> Self {
        LocateManifestError::Io(source)
    }
}

impl convert::From<string::FromUtf8Error> for LocateManifestError {
    fn from(source: string::FromUtf8Error) -> Self {
        LocateManifestError::StringConversion(source)
    }
}

impl convert::From<json::Error> for LocateManifestError {
    fn from(source: json::Error) -> Self {
        LocateManifestError::ParseJson(source)
    }
}

#[test]
fn test_manifest_path() {
    use std::path::Path;

    let manifest_path = locate_manifest().expect("failed to retrieve cargo manifest path");
    let manual_path = Path::new(file!())
        .parent()
        .unwrap()
        .join("../Cargo.toml")
        .canonicalize()
        .unwrap();
    assert_eq!(manifest_path, manual_path);
}
