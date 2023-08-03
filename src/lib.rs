use anyhow::Result;

const BINARY_BYTES: &[u8] = include_bytes!("../target/bin/gltf_validator");

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
    pub fn run(&self, path: &std::path::PathBuf) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {:?}", path));
        }

        // Shell out to gltf_validator.
        let output = std::process::Command::new(&self.installed_path)
            // This will print the validation results to stdout.
            .arg("-o")
            .arg(path)
            .output()?;

        println!("{}", String::from_utf8_lossy(&output.stdout));

        Ok(())
    }
}
