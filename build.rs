use anyhow::Result;

const GLTF_VALIDATOR_BASE_URL: &str =
    "https://github.com/KhronosGroup/glTF-Validator/releases/download";
const GLTF_VALIDATOR_VERSION: &str = "2.0.0-dev.3.8";

#[cfg(target_os = "macos")]
fn get_download_url() -> String {
    format!(
        "{}/{}/gltf_validator-{}-macos64.tar.xz",
        GLTF_VALIDATOR_BASE_URL, GLTF_VALIDATOR_VERSION, GLTF_VALIDATOR_VERSION
    )
}

#[cfg(target_os = "linux")]
fn get_download_url() -> String {
    format!(
        "{}/{}/gltf_validator-{}-linux64.tar.xz",
        GLTF_VALIDATOR_BASE_URL, GLTF_VALIDATOR_VERSION, GLTF_VALIDATOR_VERSION
    )
}

fn default_bin_dir() -> std::path::PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("target");
    path.push("bin");
    path
}

fn installed_path() -> std::path::PathBuf {
    let mut default_path = default_bin_dir();
    default_path = default_path.join("gltf_validator");
    default_path
}

/// Check if the binary has already been installed.
fn is_installed() -> bool {
    let installed_path = installed_path();
    installed_path.exists()
}

// Download the file at the given URL and save it to the given path.
fn download(file_name: &str) -> Result<std::path::PathBuf> {
    let url = get_download_url();
    println!("Downloading gltf_validator from {}", url);
    let mut response = reqwest::blocking::get(url)?;
    let tmp_path = std::env::temp_dir().join(file_name);
    let mut file = std::fs::File::create(tmp_path.clone())?;
    std::io::copy(&mut response, &mut file)?;
    Ok(tmp_path)
}

fn install() -> Result<()> {
    // Check if it was installed already.

    // If not install it.
    if !is_installed() {
        let file_name = "binary.tar.xz";
        let tmp_path = download(file_name)?;

        let bin_dir = default_bin_dir();

        // Create the directory we need to put the binary in.
        std::fs::create_dir_all(&bin_dir)?;

        // Unpack the tarball.
        let file = std::fs::File::open(&tmp_path)?;
        let mut archive = tar::Archive::new(xz::read::XzDecoder::new(file));

        // Unpack it to a temporary directory.
        let tmp_dir = tempfile::tempdir()?;
        archive.unpack(&tmp_dir)?;

        // Move the file to the bin directory.
        let bin_path = tmp_dir.path().join("gltf_validator");
        std::fs::rename(bin_path, bin_dir.join("gltf_validator"))?;

        // Delete the tmp_path tarball.
        std::fs::remove_file(tmp_path)?;
    }

    // Include the tarball in the path.
    let installed_path = default_bin_dir();

    // Set the environment variable.
    let existing_path_var = std::env::var_os("PATH").unwrap_or_default();
    let existing_paths = std::env::split_paths(&existing_path_var);
    std::env::set_var(
        "PATH",
        std::env::join_paths(existing_paths.chain(std::iter::once(installed_path)))?,
    );

    Ok(())
}

fn main() {
    // If we are on windows, exit early that it is not supported.
    if cfg!(target_os = "windows") {
        println!("cargo:warning=Windows is not supported.");
        return;
    }

    // We need to get the gltf_validator binary and put it in the path.
    install().unwrap();
}
