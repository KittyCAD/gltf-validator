use anyhow::Result;
use serde::{Deserialize, Serialize};

const BINARY_BYTES: &[u8] = include_bytes!("../target/bin/gltf_validator");

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub uri: Option<String>,
    pub mime_type: Option<MimeType>,
    pub validator_version: String,
    pub validated_at: Option<String>,
    pub issues: Issues,
    pub info: Option<Info>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum MimeType {
    #[serde(rename = "model/gltf+json")]
    ModelGltfJson,
    #[serde(rename = "model/gltf-binary")]
    ModelGltfBinary,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Issues {
    pub num_errors: u32,
    pub num_warnings: u32,
    pub num_infos: u32,
    pub num_hints: u32,
    pub messages: Vec<Message>,
    pub truncated: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub code: String,
    pub severity: Severity,
    pub pointer: Option<String>,
    pub offset: Option<u32>,
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    #[serde(rename = "0")]
    Error,
    #[serde(rename = "1")]
    Warning,
    #[serde(rename = "2")]
    Information,
    #[serde(rename = "3")]
    Hint,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub version: String,
    pub min_version: Option<String>,
    pub generator: Option<String>,
    pub extensions_used: Option<Vec<String>>,
    pub extensions_required: Option<Vec<String>>,
    pub resources: Option<Vec<Resource>>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub pointer: String,
    pub storage: Storage,
    pub mime_type: Option<String>,
    pub byte_length: Option<u32>,
    pub uri: Option<String>,
    pub image: Option<Image>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Storage {
    #[serde(rename = "data-uri")]
    DataUri,
    #[serde(rename = "buffer-view")]
    BufferView,
    #[serde(rename = "glb")]
    Glb,
    #[serde(rename = "external")]
    External,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub format: Option<Format>,
    pub primaries: Option<Primaries>,
    pub transfer: Option<Transfer>,
    pub bits: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Format {
    Rgb,
    Rgba,
    Luminance,
    LuminanceAlpha,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Primaries {
    Srgb,
    Custom,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Transfer {
    Linear,
    Srgb,
    Custom,
}

pub struct GltfValidator {
    installed_path: std::path::PathBuf,
}

impl Drop for GltfValidator {
    fn drop(&mut self) {
        // Delete the binary.
        std::fs::remove_file(&self.installed_path).unwrap();
    }
}

/// Add the `gltf_validator` binary to a temporary directory.
/// And our path.
fn init() -> Result<std::path::PathBuf> {
    let temp_dir = std::env::temp_dir();

    let installed_path = temp_dir.join("gltf_validator");

    // Write the binary bytes to the file.
    std::fs::write(&installed_path, BINARY_BYTES)?;

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
        // Deserialize, Serialize the string as our ValidationReport.
        let report: ValidationReport = serde_json::from_str(&json_string)?;

        Ok(report)
    }
}
