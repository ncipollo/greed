use std::path::PathBuf;

#[cfg(test)]
pub fn path(file: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    [manifest_dir, "src", "fixtures", file].iter().collect()
}
