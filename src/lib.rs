//! A rust library and binary wrapper around the Khronos group
//! [glTF-Validator](https://github.com/KhronosGroup/glTF-Validator) tool.
//!
//! Use it like this to validate a glTF file:
//! ```rust
//! use gltf_validator::GltfValidator;
//!
//! let validator = GltfValidator::new().unwrap();
//! let report = validator.run(&std::path::PathBuf::from("tests/cube.glb")).unwrap();
//! assert_eq!(report.issues.num_errors, 0);
//! ```

#![deny(missing_docs)]

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Represents the validation report.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    /// URI of validated asset.
    pub uri: Option<String>,
    /// MIME type of validated asset.
    pub mime_type: Option<MimeType>,
    /// Version string of glTF-Validator.
    pub validator_version: String,
    /// UTC timestamp of validation time.
    pub validated_at: Option<String>,
    /// Validation issues.
    pub issues: Issues,
    /// Information about the validated asset.
    pub info: Option<Info>,
}

/// Possible MIME types.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum MimeType {
    /// glTF asset in plain text form.
    #[serde(rename = "model/gltf+json")]
    ModelGltfJson,
    /// glTF asset in GLB container.
    #[serde(rename = "model/gltf-binary")]
    ModelGltfBinary,
}

/// Represents validation issues.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Issues {
    /// Number of errors.
    pub num_errors: u32,
    /// Number of warnings.
    pub num_warnings: u32,
    /// Number of informational messages.
    pub num_infos: u32,
    /// Number of hints.
    pub num_hints: u32,
    /// Array of message objects.
    pub messages: Vec<Message>,
    /// Indicates if validation output is incomplete due to too many messages.
    pub truncated: bool,
}

/// Represents a validation message.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// Message code.
    pub code: String,
    /// Severity of the message.
    pub severity: Severity,
    /// JSON Pointer to the object causing the issue.
    pub pointer: Option<String>,
    /// Byte offset in GLB file. Applicable only to GLB issues.
    pub offset: Option<u32>,
    /// Actual message string.
    pub message: String,
}

/// Possible severities for validation messages.
#[derive(Debug, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
pub enum Severity {
    /// Error severity.
    Error = 0,
    /// Warning severity.
    Warning = 1,
    /// Information severity.
    Information = 2,
    /// Hint severity.
    Hint = 3,
}

/// Information about the validated asset.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    /// The glTF version that this asset targets.
    pub version: String,
    /// The minimum glTF version that this asset targets.
    pub min_version: Option<String>,
    /// Tool that generated this glTF model.
    pub generator: Option<String>,
    /// Names of glTF extensions used somewhere in this asset.
    pub extensions_used: Option<Vec<String>>,
    /// Names of glTF extensions required to properly load this asset.
    pub extensions_required: Option<Vec<String>>,
    /// Details about resources used in the asset.
    pub resources: Option<Vec<Resource>>,
}

/// Represents a resource used in the asset.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    /// JSON Pointer to the resource.
    pub pointer: String,
    /// How the resource is stored.
    pub storage: Storage,
    /// Mime type of the resource.
    pub mime_type: Option<String>,
    /// Byte length of the resource.
    pub byte_length: Option<u32>,
    /// URI of the resource.
    pub uri: Option<String>,
    /// Image-specific metadata.
    pub image: Option<Image>,
}

/// Possible ways a resource can be stored.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Storage {
    /// Resource is stored as Data-URI.
    #[serde(rename = "data-uri")]
    DataUri,
    /// Resource is stored within glTF buffer and accessed via bufferView.
    #[serde(rename = "buffer-view")]
    BufferView,
    /// Resource is stored in binary chunk of GLB container.
    #[serde(rename = "glb")]
    Glb,
    /// Resource is stored externally.
    #[serde(rename = "external")]
    External,
}

/// Image-specific metadata.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// Width of the image.
    pub width: u32,
    /// Height of the image.
    pub height: u32,
    /// Format of the image.
    pub format: Option<Format>,
    /// Primary colors of the image.
    pub primaries: Option<Primaries>,
    /// Transfer function of the image.
    pub transfer: Option<Transfer>,
    /// Bit depth of the image.
    pub bits: Option<u32>,
}

/// Possible formats for images.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Format {
    /// RGB format.
    Rgb,
    /// RGBA format.
    Rgba,
    /// Luminance format.
    Luminance,
    /// Luminance-Alpha format.
    LuminanceAlpha,
}

/// Possible primary colors for images.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Primaries {
    /// sRGB primary colors.
    Srgb,
    /// Custom primary colors.
    Custom,
}

/// Possible transfer functions for images.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Transfer {
    /// Linear transfer function.
    Linear,
    /// sRGB transfer function.
    Srgb,
    /// Custom transfer function.
    Custom,
}

const BINARY_BYTES: &[u8] = include_bytes!("../target/bin/gltf_validator");

/// An instance of glTF validator.
pub struct GltfValidator {
    installed_path: std::path::PathBuf,
}

// impl Drop for GltfValidator {
//     fn drop(&mut self) {
//         // Delete the binary.
//         std::fs::remove_file(&self.installed_path).unwrap();
//     }
// }

/// Add the `gltf_validator` binary to a temporary directory.
/// And our path.
fn init() -> Result<std::path::PathBuf> {
    use std::io::Write;

    let temp_dir = std::env::temp_dir();

    let installed_path = temp_dir.join("gltf_validator");

    // Write the binary bytes to the file.
    let mut file = std::fs::File::create(&installed_path)?;
    file.write_all(BINARY_BYTES)?;
    file.flush()?;

    // Make sure the file is executable.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&installed_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&installed_path, perms)?;
    }

    let existing_path_var = std::env::var_os("PATH").unwrap_or_default();
    let existing_paths = std::env::split_paths(&existing_path_var);
    std::env::set_var(
        "PATH",
        std::env::join_paths(existing_paths.chain(std::iter::once(installed_path.clone())))?,
    );

    Ok(installed_path)
}

impl GltfValidator {
    /// Create a new instance of the validator.
    pub fn new() -> Result<Self> {
        let installed_path = init()?;
        Ok(Self { installed_path })
    }

    /// Run gltf-validator on a specific file.
    pub fn run(&self, path: &std::path::PathBuf) -> Result<ValidationReport> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {:?}", path));
        }

        // Shell out to gltf_validator.
        let output = std::process::Command::new(&self.installed_path)
            // This will print the validation results to stdout.
            .arg("-o")
            .arg(path)
            .output()?;

        let json_string = String::from_utf8_lossy(&output.stdout);

        // Deserialize the string as our ValidationReport.
        let report: ValidationReport = serde_json::from_str(&json_string)?;

        Ok(report)
    }
}
