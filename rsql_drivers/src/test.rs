use std::path;
use std::path::PathBuf;

/// Returns the url to the specified dataset file.
pub(crate) fn dataset_url<S: AsRef<str>>(scheme: S, file_name: S) -> String {
    let scheme = scheme.as_ref();
    let file_name = file_name.as_ref();
    let crate_directory = env!("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from(crate_directory);
    path.push("..");
    path.push("datasets");
    path.push(file_name);

    let dataset_path = path
        .to_string_lossy()
        .to_string()
        .replace(path::MAIN_SEPARATOR, "/");
    #[cfg(target_os = "windows")]
    let dataset_path = format!("/{dataset_path}");

    format!("{scheme}://{dataset_path}")
}
