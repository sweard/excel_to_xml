use std::{
    ffi::OsStr,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

use crate::error::{ConversionError, ConversionResult};

fn sorted_entries(path: &Path) -> ConversionResult<Vec<DirEntry>> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| {
            ConversionError::ResourceDiscovery(format!("无法读取目录 {}: {error}", path.display()))
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            ConversionError::ResourceDiscovery(format!("无法遍历目录 {}: {error}", path.display()))
        })?;
    entries.sort_by_key(DirEntry::file_name);
    Ok(entries)
}

fn is_ignored(path: &Path, ignore: &[String]) -> bool {
    path.components().any(|component| {
        ignore
            .iter()
            .any(|ignored| component.as_os_str() == OsStr::new(ignored))
    })
}

fn is_real_directory(path: &Path) -> ConversionResult<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.file_type().is_dir()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(ConversionError::ResourceDiscovery(format!(
            "无法读取路径 {}: {error}",
            path.display()
        ))),
    }
}

/// 以稳定顺序查找第一个目标文件夹，不跟随符号链接。
pub fn find_target_folder(
    input: &Path,
    target: &str,
    ignore: &[String],
) -> ConversionResult<Option<PathBuf>> {
    if !is_real_directory(input)? || is_ignored(input, ignore) {
        return Ok(None);
    }

    if input.file_name() == Some(OsStr::new(target)) {
        return Ok(Some(input.to_path_buf()));
    }

    for entry in sorted_entries(input)? {
        if !entry
            .file_type()
            .map_err(|error| {
                ConversionError::ResourceDiscovery(format!(
                    "无法读取目录项 {}: {error}",
                    entry.path().display()
                ))
            })?
            .is_dir()
        {
            continue;
        }
        if let Some(found) = find_target_folder(&entry.path(), target, ignore)? {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

/// 收集 `values` 与 `values-*` 目录中的精确目标文件，结果按路径排序。
pub fn collect_target_files(
    path: &Path,
    target_dir_name: &str,
    target: &str,
) -> ConversionResult<Vec<PathBuf>> {
    if !is_real_directory(path)? {
        return Err(ConversionError::ResourceDiscovery(format!(
            "资源路径不是目录: {}",
            path.display()
        )));
    }

    let qualified_prefix = format!("{target_dir_name}-");
    let mut result = Vec::new();
    for entry in sorted_entries(path)? {
        if !entry
            .file_type()
            .map_err(|error| {
                ConversionError::ResourceDiscovery(format!(
                    "无法读取目录项 {}: {error}",
                    entry.path().display()
                ))
            })?
            .is_dir()
        {
            continue;
        }
        let directory_name = entry.file_name();
        let directory_name = directory_name.to_string_lossy();
        if directory_name != target_dir_name && !directory_name.starts_with(&qualified_prefix) {
            continue;
        }

        let target_path = entry.path().join(target);
        match fs::symlink_metadata(&target_path) {
            Ok(metadata) if metadata.file_type().is_file() => {
                log::debug!("找到目标文件: {}", target_path.display());
                result.push(target_path);
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(ConversionError::ResourceDiscovery(format!(
                    "无法读取目标文件 {}: {error}",
                    target_path.display()
                )));
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_DIR_ID: AtomicU64 = AtomicU64::new(0);

    fn test_directory(name: &str) -> PathBuf {
        let id = TEST_DIR_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "excel-to-xml-find-{name}-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create test directory");
        path
    }

    #[test]
    fn ignores_exact_path_components_and_uses_sorted_order() {
        let root = test_directory("ignore");
        let expected = root.join("a-build-variant/res");
        fs::create_dir_all(&expected).expect("create expected res");
        fs::create_dir_all(root.join("build/res")).expect("create ignored res");

        let found = find_target_folder(&root, "res", &["build".to_string()])
            .expect("find succeeds")
            .expect("res exists");

        assert_eq!(found, expected);
        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn collects_only_exact_target_files_from_values_directories() {
        let root = test_directory("collect");
        for directory in ["values-zh", "values", "my-values", "valuesBackup"] {
            fs::create_dir_all(root.join(directory)).expect("create values directory");
        }
        fs::write(root.join("values/strings.xml"), "default").expect("write default");
        fs::write(root.join("values-zh/strings.xml"), "zh").expect("write zh");
        fs::write(root.join("my-values/strings.xml"), "wrong").expect("write wrong dir");
        fs::write(root.join("valuesBackup/strings.xml"), "wrong").expect("write backup dir");
        fs::write(root.join("values/not_strings.xml"), "wrong").expect("write wrong file");

        let files =
            collect_target_files(&root, "values", "strings.xml").expect("collection succeeds");

        assert_eq!(
            files,
            vec![
                root.join("values/strings.xml"),
                root.join("values-zh/strings.xml")
            ]
        );
        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[cfg(unix)]
    #[test]
    fn does_not_follow_directory_symlinks() {
        use std::os::unix::fs::symlink;

        let root = test_directory("symlink");
        symlink(&root, root.join("a-cycle")).expect("create cycle symlink");
        let expected = root.join("b-module/res");
        fs::create_dir_all(&expected).expect("create real res");

        let found = find_target_folder(&root, "res", &[])
            .expect("find succeeds")
            .expect("res exists");

        assert_eq!(found, expected);
        fs::remove_dir_all(root).expect("remove test directory");
    }
}
