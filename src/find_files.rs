use std::{fs::read_dir, path::Path};

/**
 * 查找目标文件
 * @param path 路径
 * @param target_dir_name 目标文件夹名称
 * @param target 目标文件名称
 */
pub fn collect_target_files(path: &str, target_dir_name:&str, target: &str) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    let path = Path::new(path);
    if !path.is_dir() {
        // 当前路径不是文件夹
        return files;
    }

    // 文件夹下的子项目列表
    let entries = match read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return files,
    };

    // 遍历子项目
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            // 不是文件夹，跳过
            continue;
        }

        let dir_name = match entry.file_name().to_str() {
            Some(name) => name.to_owned(),
            None => continue,
        };

        if !dir_name.contains(target_dir_name) {
            // 文件夹名称不包含目标文件夹名称，跳过
            continue;
        }

        // 文件夹下，子文件列表
        let file_names = match read_dir(entry_path) {
            Ok(file_names) => file_names,
            Err(_) => continue,
        };

        // 找到第一个符合条件的文件
        file_names.flatten().find(|file_name| {
            let file_path = file_name.path();
            let abs_path = file_path.to_str(); 
            match abs_path {
                Some(value) => {
                    let result = value.ends_with(target) && file_path.is_file();
                    if result {
                        println!("找到目标文件: {}", value);
                        files.push(value.to_string());
                    }
                    result
                },
                None => false,
            }
        }); 
    }
    files
}

/**
 * 查找目标文件夹
 * @param input 输入路径
 * @param target 目标文件夹名称
 * @param ignore 忽略的文件夹
 */
pub fn find_target_folder(input: &str, target: &str, ignore: &Vec<&str>) -> Option<String> {
    let path = Path::new(input);
    if !path.is_dir() {
        return None;
    }

    if path.file_name().unwrap_or_default() == target {
        if ignore.iter().any(|&x| input.contains(x)) {
            return None;
        }
        return Some(input.to_string());
    }

    let entries = match read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return None,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        let entry_path = entry_path.to_str().unwrap_or("");
        if let Some(found) = find_target_folder(entry_path, target, ignore) {
            return Some(found);
        }
    }
    None
}