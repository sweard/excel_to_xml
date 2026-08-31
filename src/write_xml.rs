use std::{
    collections::{HashMap, HashSet},
    fs::{copy, remove_file, rename, File},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    config::ParsedCfg,
    error::{ConversionError, ConversionResult},
    find_files, read_excel,
};
use calamine::Xlsx;
use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
    Writer,
};
use regex::Regex;

const XML_NAME: &str = "name";
// 格式化相关常量
const XML_NEWLINE: &str = "\n";
const XML_INDENT: &str = "\n    ";
const XML_STRING: &str = "string";
const XML_RESOURCES: &str = "resources";

#[derive(Default)]
struct PathIndex {
    path: PathBuf,
    index: u32,
    lang: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversionMode {
    LowMemory,
    Fast,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConversionReport {
    pub files_updated: usize,
}

struct PreparedConversion {
    workbook: Xlsx<BufReader<File>>,
    parsed_cfg: ParsedCfg,
    targets: Vec<PathIndex>,
    regex: Option<Regex>,
}

static STAGED_FILE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
struct StagedFile {
    original_path: PathBuf,
    temp_path: PathBuf,
    backup_path: PathBuf,
}

impl StagedFile {
    fn new(path: &Path) -> Self {
        let id = STAGED_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let suffix = format!("{}-{}", std::process::id(), id);
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        Self {
            original_path: path.to_path_buf(),
            temp_path: path.with_file_name(format!("{file_name}.e2xf-{suffix}.tmp")),
            backup_path: path.with_file_name(format!("{file_name}.e2xf-{suffix}.bak")),
        }
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        if self.temp_path.exists() {
            let _ = remove_file(&self.temp_path);
        }
    }
}

fn remove_backups(staged_files: &[StagedFile]) {
    for staged in staged_files {
        if staged.backup_path.exists() {
            if let Err(err) = remove_file(&staged.backup_path) {
                log::warn!(
                    "清理备份文件失败: {}, err: {}",
                    staged.backup_path.display(),
                    err
                );
            }
        }
    }
}

fn restore_backups(staged_files: &[StagedFile]) -> Vec<String> {
    let mut errors = Vec::new();
    for staged in staged_files.iter().rev() {
        if !staged.backup_path.exists() {
            continue;
        }
        if let Err(err) = copy(&staged.backup_path, &staged.original_path) {
            errors.push(format!("{}: {}", staged.original_path.display(), err));
        }
    }
    errors
}

#[cfg(not(windows))]
fn promote_staged_file(staged: &StagedFile) -> io::Result<()> {
    // 临时文件与原文件位于同一目录，rename 会原子替换目标文件。
    rename(&staged.temp_path, &staged.original_path)
}

#[cfg(windows)]
fn promote_staged_file(staged: &StagedFile) -> io::Result<()> {
    // std::fs::rename 在 Windows 上不能覆盖已有文件，备份已在删除前创建。
    remove_file(&staged.original_path)?;
    rename(&staged.temp_path, &staged.original_path)
}

fn commit_staged_files(staged_files: &[StagedFile]) -> ConversionResult<()> {
    // 所有文件均成功生成后再建立备份，避免解析/写入失败时触碰原文件。
    for staged in staged_files {
        if let Err(err) = copy(&staged.original_path, &staged.backup_path) {
            remove_backups(staged_files);
            return Err(ConversionError::Transaction(format!(
                "创建备份文件失败: {}: {}",
                staged.backup_path.display(),
                err
            )));
        }
    }

    for (index, staged) in staged_files.iter().enumerate() {
        if let Err(err) = promote_staged_file(staged) {
            let rollback_errors = restore_backups(&staged_files[..=index]);
            if rollback_errors.is_empty() {
                remove_backups(staged_files);
                return Err(ConversionError::Transaction(format!(
                    "替换文件失败，已回滚: {}: {}",
                    staged.original_path.display(),
                    err
                )));
            }
            return Err(ConversionError::Transaction(format!(
                "替换文件失败且回滚不完整: {}: {}; 回滚失败: {}; 备份文件已保留",
                staged.original_path.display(),
                err,
                rollback_errors.join("; ")
            )));
        }
    }

    remove_backups(staged_files);
    Ok(())
}

fn resolve_target_paths(
    parsed_cfg: &ParsedCfg,
    paths: &[PathBuf],
) -> ConversionResult<Vec<PathIndex>> {
    if parsed_cfg.lang_index_map.is_empty() {
        return Err(ConversionError::TargetResolution(
            "Excel 中没有匹配配置 langMap 的语言列".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(parsed_cfg.lang_index_map.len());
    for (lang, index) in &parsed_cfg.lang_index_map {
        let end_point = if lang == &parsed_cfg.default_lang {
            PathBuf::from("values/strings.xml")
        } else {
            PathBuf::from(format!("values-{lang}/strings.xml"))
        };
        let path = paths
            .iter()
            .find(|path| path.ends_with(&end_point))
            .ok_or_else(|| {
                ConversionError::TargetResolution(format!(
                    "缺少语言 {lang} 的目标文件: {}",
                    end_point.display()
                ))
            })?;
        result.push(PathIndex {
            path: path.clone(),
            index: *index,
            lang: lang.clone(),
        });
    }

    if let Some(pos) = result
        .iter()
        .position(|item| item.lang == parsed_cfg.default_lang)
    {
        result.swap(0, pos);
    }
    Ok(result)
}

fn prepare_conversion(
    cfg_json: &str,
    excel_path: &str,
    xml_dir_path: &str,
) -> ConversionResult<PreparedConversion> {
    let mut workbook = read_excel::open_excel_workbook(excel_path)?;
    let parsed_cfg =
        read_excel::parse_cfg_with_workbook(&mut workbook, cfg_json).inspect_err(|error| {
            log::error!("解析配置时出错: {error}");
        })?;
    log::info!("解析配置成功: {:?}", parsed_cfg);
    let folder = find_files::find_target_folder(
        std::path::Path::new(xml_dir_path),
        "res",
        &parsed_cfg.ignore_folder,
    )?;
    let Some(res_folder) = folder else {
        log::error!("未找到res文件夹");
        return Err(ConversionError::ResourceDiscovery(
            "未找到res文件夹".to_string(),
        ));
    };
    log::info!("找到res文件夹: {}", res_folder.display());
    let paths = find_files::collect_target_files(&res_folder, "values", "strings.xml")?;
    let targets = resolve_target_paths(&parsed_cfg, &paths)?;
    let regex = if is_blank(&parsed_cfg.regex) {
        None
    } else {
        Some(
            Regex::new(&parsed_cfg.regex)
                .map_err(|error| ConversionError::InvalidRegex(error.to_string()))?,
        )
    };
    Ok(PreparedConversion {
        workbook,
        parsed_cfg,
        targets,
        regex,
    })
}

fn stage_low_memory(prepared: &mut PreparedConversion) -> ConversionResult<Vec<StagedFile>> {
    // 同一个HashMap在各语言之间循环复用。
    let mut tag_value_map = HashMap::new();
    let mut staged_files = Vec::with_capacity(prepared.targets.len());
    let mut default_valug_map: HashMap<String, String> = HashMap::new();
    let default_lang = &prepared.parsed_cfg.default_lang;
    let replace_blank_with_default = prepared.parsed_cfg.replace_blank_with_default;

    // 遍历语言索引映射
    for target in &prepared.targets {
        let lang = &target.lang;
        let is_default_lang = lang == default_lang;

        // 清空map，准备复用
        tag_value_map.clear();

        // 构建标签值映射 - 复用workbook、sheet_name和tag_value_map
        let lang_index = target.index;
        read_excel::process_excel_single_lang(
            &mut prepared.workbook,
            &prepared.parsed_cfg.sheet_name,
            prepared.parsed_cfg.tag_index,
            lang_index,
            &mut tag_value_map,
        )?;
        // 如果是默认语言，将tag_value_map的内容复制到default_valug_map
        // 以便后续处理空值
        if is_default_lang && replace_blank_with_default {
            // 处理默认语言,使用空default_valug_map
            staged_files.push(update_xml_file(
                &target.path,
                &tag_value_map,
                &default_valug_map,
                &prepared.parsed_cfg,
                prepared.regex.as_ref(),
            )?);
            default_valug_map = tag_value_map.clone();
        } else {
            // 处理XML文件
            staged_files.push(update_xml_file(
                &target.path,
                &tag_value_map,
                &default_valug_map,
                &prepared.parsed_cfg,
                prepared.regex.as_ref(),
            )?);
        }
    }
    Ok(staged_files)
}

fn stage_fast(prepared: &mut PreparedConversion) -> ConversionResult<Vec<StagedFile>> {
    let mut staged_files = Vec::with_capacity(prepared.targets.len());
    let default_lang = &prepared.parsed_cfg.default_lang;
    let replace_blank_with_default = prepared.parsed_cfg.replace_blank_with_default;
    let mut default_valug_map: HashMap<String, String> = HashMap::new();
    let lang_index_vec: Vec<u32> = prepared.targets.iter().map(|p| p.index).collect();
    let mut translation_rows = Vec::new();

    read_excel::process_excel_multi_lang(
        &mut prepared.workbook,
        &prepared.parsed_cfg.sheet_name,
        prepared.parsed_cfg.tag_index,
        &lang_index_vec,
        &mut translation_rows,
    )?;
    log::debug!("translation row count: {}", translation_rows.len());

    // 处理XML文件
    for (language_slot, p) in prepared.targets.iter().enumerate() {
        let index = p.index;
        let path = &p.path;
        let lang = &p.lang;
        let mut write_map = HashMap::with_capacity(translation_rows.len());
        for row in &translation_rows {
            write_map.insert(row.tag.clone(), row.values[language_slot].clone());
        }

        let is_default_lang = lang == default_lang;
        // 如果是默认语言，将tag_value_map的内容复制到default_valug_map
        // 以便后续处理空值
        let res = if is_default_lang && replace_blank_with_default {
            // 处理默认语言,使用空default_valug_map
            let res = update_xml_file(
                path,
                &write_map,
                &default_valug_map,
                &prepared.parsed_cfg,
                prepared.regex.as_ref(),
            );
            // 处理完后，默认语言的值会被复制到default_valug_map
            default_valug_map = write_map.clone();
            res
        } else {
            update_xml_file(
                path,
                &write_map,
                &default_valug_map,
                &prepared.parsed_cfg,
                prepared.regex.as_ref(),
            )
        };
        match res {
            Ok(staged) => staged_files.push(staged),
            Err(err) => {
                log::error!("更新XML文件失败,lang index: {}, err: {:?}", index, err);
                return Err(err);
            }
        }
    }
    Ok(staged_files)
}

pub fn convert(
    cfg_json: &str,
    excel_path: &str,
    xml_dir_path: &str,
    mode: ConversionMode,
) -> ConversionResult<ConversionReport> {
    let mut prepared = prepare_conversion(cfg_json, excel_path, xml_dir_path)?;
    let staged_files = match mode {
        ConversionMode::LowMemory => stage_low_memory(&mut prepared)?,
        ConversionMode::Fast => stage_fast(&mut prepared)?,
    };
    let report = ConversionReport {
        files_updated: staged_files.len(),
    };
    commit_staged_files(&staged_files)?;
    Ok(report)
}

/// 更新XML文件
fn update_xml_file(
    path: &Path,
    tag_value_map: &HashMap<String, String>,
    default_valug_map: &HashMap<String, String>,
    parsed_cfg: &ParsedCfg,
    regex: Option<&Regex>,
) -> ConversionResult<StagedFile> {
    let staged_file = StagedFile::new(path);

    // 打开原始XML文件和临时文件
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut xml_reader = quick_xml::Reader::from_reader(reader);

    let output_file = File::create(&staged_file.temp_path)?;
    let writer = BufWriter::new(output_file);
    let mut xml_writer = Writer::new(writer);

    let mut buf = Vec::with_capacity(4 * 1024);
    let mut current_tag_name = None;
    let mut updated_tags = HashSet::new();

    let replace_blank_with_default = parsed_cfg.replace_blank_with_default;
    let disable_escape = parsed_cfg.disable_escape;
    let escape_only = &parsed_cfg.escape_only;
    if parsed_cfg.reset {
        // 重写XML文件
        // 1. 写入XML声明
        xml_writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

        // 2. 添加换行
        xml_writer.write_event(Event::Text(BytesText::new(XML_NEWLINE)))?;

        // 3. 打开resources标签
        xml_writer.write_event(Event::Start(BytesStart::new(XML_RESOURCES)))?;

        // 4. 按照tag_value_map写入所有标签
        for (tag, value) in sorted_tag_values(tag_value_map) {
            // 添加换行和缩进
            xml_writer.write_event(Event::Text(BytesText::new(XML_INDENT)))?;

            // 创建string标签
            let mut elem = BytesStart::new(XML_STRING);
            elem.push_attribute((XML_NAME, tag.as_str()));
            xml_writer.write_event(Event::Start(elem))?;
            let write_value =
                get_write_value(tag, value, default_valug_map, replace_blank_with_default);
            write_text(
                disable_escape,
                &mut xml_writer,
                write_value,
                escape_only,
                regex,
            )?;
            xml_writer.write_event(Event::End(BytesEnd::new(XML_STRING)))?;
        }

        // 5. 添加最后的换行
        xml_writer.write_event(Event::Text(BytesText::new(XML_NEWLINE)))?;

        // 6. 关闭resources标签
        xml_writer.write_event(Event::End(BytesEnd::new(XML_RESOURCES)))?;
    } else {
        // 更新XML文件
        let mut open_element_depth = 0usize;
        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    open_element_depth += 1;
                    if e.name().as_ref() == XML_STRING {
                        // 提取name属性
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == XML_NAME {
                                let tag_name = attr.value.as_ref();
                                if tag_value_map.contains_key(tag_name) {
                                    current_tag_name = Some(tag_name.to_string());
                                }
                            }
                        }
                    }
                    xml_writer.write_event(Event::Start(e.to_owned()))?;
                }

                Ok(Event::End(ref e)) => {
                    if open_element_depth == 0 {
                        return Err(ConversionError::Xml(
                            "XML 结束标签没有对应的开始标签".to_string(),
                        ));
                    }
                    if e.name().as_ref() == XML_RESOURCES {
                        // 在关闭resources标签前添加缺失的标签
                        add_missing_tags(
                            &mut xml_writer,
                            tag_value_map,
                            default_valug_map,
                            &updated_tags,
                            parsed_cfg,
                            regex,
                        )?;
                    }
                    xml_writer.write_event(Event::End(e.to_owned()))?;
                    open_element_depth -= 1;
                }

                Ok(Event::Text(e)) => {
                    // 提前处理没有当前标签的情况
                    if current_tag_name.is_none() {
                        xml_writer.write_event(Event::Text(e.to_owned()))?;
                        continue;
                    }

                    let tag_name = current_tag_name.as_ref().unwrap();
                    updated_tags.insert(tag_name.to_string());

                    // 更新文本内容
                    match tag_value_map.get(tag_name) {
                        Some(value) => {
                            let write_value = get_write_value(
                                tag_name,
                                value,
                                default_valug_map,
                                replace_blank_with_default,
                            );
                            let write_value_empty = write_value.trim().is_empty();
                            let mut keep_original = false;
                            if write_value_empty {
                                let original_text_empty = quick_xml::escape::unescape(e.as_ref())
                                    .ok()
                                    .map(|text| text.trim().is_empty())
                                    .unwrap_or(false);
                                if write_value_empty && original_text_empty {
                                    // fixme 内容均为空，不做修改, 会有不换行的问题
                                    keep_original = true;
                                }
                            }
                            if keep_original {
                                xml_writer.write_event(Event::Text(e.to_owned()))?;
                            } else {
                                write_text(
                                    disable_escape,
                                    &mut xml_writer,
                                    write_value,
                                    escape_only,
                                    regex,
                                )?;
                            }
                        }
                        _ => {
                            xml_writer.write_event(Event::Text(e.to_owned()))?;
                        }
                    }
                    current_tag_name = None;
                }

                Ok(Event::Eof) => {
                    if open_element_depth != 0 {
                        return Err(ConversionError::Xml(
                            "XML 文件在标签闭合前意外结束".to_string(),
                        ));
                    }
                    break;
                }
                Ok(e) => xml_writer.write_event(e)?,
                Err(e) => return Err(e.into()),
            }
            buf.clear();
        }
    }
    let mut buffered_writer = xml_writer.into_inner();
    buffered_writer.flush()?;
    buffered_writer.get_ref().sync_all()?;
    drop(buffered_writer);
    drop(xml_reader);
    Ok(staged_file)
}

/// 添加缺失的标签
fn add_missing_tags(
    xml_writer: &mut Writer<BufWriter<File>>,
    tag_value_map: &HashMap<String, String>,
    default_valug_map: &HashMap<String, String>,
    updated_tags: &HashSet<String>,
    parsed_cfg: &ParsedCfg,
    regex: Option<&Regex>,
) -> ConversionResult<()> {
    let disable_escape = parsed_cfg.disable_escape;
    let escape_only = &parsed_cfg.escape_only;
    let mut missing_tag_added = false;
    for (tag, value) in sorted_tag_values(tag_value_map) {
        if !updated_tags.contains(tag) {
            if !missing_tag_added {
                // 结束时，需要添加换行和缩进
                missing_tag_added = true;
            }
            // 为每个新标签添加缩进
            xml_writer.write_event(Event::Text(BytesText::new(XML_INDENT)))?;

            // 创建新标签
            let mut elem = BytesStart::new(XML_STRING);
            elem.push_attribute((XML_NAME, tag.as_str()));

            xml_writer.write_event(Event::Start(elem))?;
            let write_value = get_write_value(
                tag,
                value,
                default_valug_map,
                parsed_cfg.replace_blank_with_default,
            );
            write_text(disable_escape, xml_writer, write_value, escape_only, regex)?;
            xml_writer.write_event(Event::End(BytesEnd::new(XML_STRING)))?;
        }
    }
    if missing_tag_added {
        // 如果写入了新tag，添加换行和缩进
        xml_writer.write_event(Event::Text(BytesText::new(XML_NEWLINE)))?;
    }
    Ok(())
}

fn write_text(
    disable_escape: bool,
    xml_writer: &mut Writer<BufWriter<File>>,
    value: &str,
    escape_only: &[(String, String)],
    regex: Option<&Regex>,
) -> ConversionResult<()> {
    let value = if let Some(regex) = regex {
        // 使用正则表达式替换
        regex.replace_all(value, "").to_string()
    } else {
        value.to_string()
    };
    if disable_escape {
        // 直接写入文本内容，不会再自动转义
        xml_writer.write_event(Event::Text(BytesText::from_escaped(value)))?;
    } else {
        if escape_only.is_empty() {
            // 转义所有内容
            xml_writer.write_event(Event::Text(BytesText::new(&value)))?;
        } else {
            // 只转义指定的内容
            let mut escaped_value = value.to_string();
            for (key, val) in escape_only {
                escaped_value = escaped_value.replace(key, val);
            }
            xml_writer.write_event(Event::Text(BytesText::from_escaped(&escaped_value)))?;
        }
    }
    Ok(())
}

fn sorted_tag_values(tag_value_map: &HashMap<String, String>) -> Vec<(&String, &String)> {
    let mut entries = tag_value_map.iter().collect::<Vec<_>>();
    entries.sort_unstable_by_key(|(tag, _)| *tag);
    entries
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn get_write_value<'a>(
    tag: &String,
    value: &'a String,
    default_valug_map: &'a HashMap<String, String>,
    replace_blank_with_default: bool,
) -> &'a String {
    if is_blank(value) && !default_valug_map.is_empty() && replace_blank_with_default {
        // 如果值为空，尝试使用默认语言的值替换
        match default_valug_map.get(tag) {
            Some(default_value) => default_value,
            None => value,
        }
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_values_are_sorted_by_tag() {
        let tag_value_map = HashMap::from([
            ("z_tag".to_string(), "last".to_string()),
            ("a_tag".to_string(), "first".to_string()),
        ]);

        let tags = sorted_tag_values(&tag_value_map)
            .into_iter()
            .map(|(tag, _)| tag.as_str())
            .collect::<Vec<_>>();

        assert_eq!(tags, vec!["a_tag", "z_tag"]);
    }

    #[test]
    fn commit_failure_restores_already_replaced_files() {
        let id = STAGED_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("excel-to-xml-rollback-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&root).expect("create rollback test directory");
        let first_path = root.join("first.xml");
        let second_path = root.join("second.xml");
        std::fs::write(&first_path, "old first").expect("write first original");
        std::fs::write(&second_path, "old second").expect("write second original");

        let first = StagedFile::new(&first_path);
        let second = StagedFile::new(&second_path);
        std::fs::write(&first.temp_path, "new first").expect("write first staged file");
        // second.temp_path deliberately does not exist, forcing promotion to fail
        // after the first file has already been replaced.
        let staged_files = vec![first, second];

        let error = commit_staged_files(&staged_files).expect_err("second promotion must fail");

        assert!(error.to_string().contains("已回滚"), "{error}");
        assert_eq!(
            std::fs::read_to_string(&first_path).expect("read first after rollback"),
            "old first"
        );
        assert_eq!(
            std::fs::read_to_string(&second_path).expect("read second after rollback"),
            "old second"
        );
        assert!(staged_files
            .iter()
            .all(|staged| !staged.backup_path.exists()));

        drop(staged_files);
        std::fs::remove_dir_all(root).expect("remove rollback test directory");
    }
}
