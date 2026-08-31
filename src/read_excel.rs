use crate::config::ParsedCfg;
use calamine::{open_workbook, DataType, Reader, Xlsx};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;

/// 自定义错误类型，用于更清晰的错误处理
#[derive(Debug)]
pub enum ExcelError {
    NoSheetsFound,
    InvalidFirstLine,
    TagNotFound(String),
    CellConversionFailed(String),
}

impl fmt::Display for ExcelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExcelError::NoSheetsFound => write!(f, "未找到任何工作表"),
            ExcelError::InvalidFirstLine => write!(f, "工作表为空"),
            ExcelError::TagNotFound(tag) => write!(f, "未找到标签: {}", tag),
            ExcelError::CellConversionFailed(msg) => write!(f, "Excel 格式错误: {}", msg),
        }
    }
}

impl Error for ExcelError {}

/// 读取Excel文件并解析
/// 单行单语言数据结构
#[derive(Default)]
struct RowSingleLangData {
    row: Option<u32>,
    tag: Option<String>,
    value: Option<String>,
}

/// 读取Excel文件并解析
/// 单行多语言数据结构
struct RowMultiLangData {
    row: Option<u32>,
    tag: Option<String>,
    values: Vec<String>,
}

impl RowMultiLangData {
    fn new(language_count: usize) -> Self {
        Self {
            row: None,
            tag: None,
            values: vec![String::new(); language_count],
        }
    }
}

pub(crate) struct TranslationRow {
    pub(crate) tag: String,
    pub(crate) values: Vec<String>,
}

fn push_translation_row(cur: &mut RowMultiLangData, rows: &mut Vec<TranslationRow>) {
    let Some(tag) = cur.tag.take() else {
        return;
    };
    let tag = tag.trim();
    if tag.is_empty() {
        return;
    }
    rows.push(TranslationRow {
        tag: tag.to_string(),
        values: std::mem::take(&mut cur.values),
    });
}

/**
 * 解析Excel文件
 * @param workbook 已打开的Excel工作簿
 * @param config_json 用户输入的配置JSON
 * @return 解析后的配置
 */
pub fn parse_cfg_with_workbook(
    workbook: &mut Xlsx<BufReader<File>>,
    config_json: &str,
) -> Result<ParsedCfg, Box<dyn Error>> {
    log::info!("开始解析Excel配置: {}", config_json);
    // 解析配置JSON
    let mut parsed_cfg = ParsedCfg::from_json(config_json)?;

    let sheet_name = if parsed_cfg.sheet_name.is_empty() {
        workbook
            .sheet_names()
            .first()
            .cloned()
            .ok_or_else(|| Box::new(ExcelError::NoSheetsFound))?
    } else {
        parsed_cfg.sheet_name.clone()
    };
    parsed_cfg.sheet_name = sheet_name.clone();

    // 获取cell_reader迭代器
    let cell_reader = workbook.worksheet_cells_reader(&sheet_name);
    // 检查是否成功获取cell_reader
    let mut cell_reader = match cell_reader {
        Ok(reader) => reader,
        Err(_) => {
            return Err(Box::new(ExcelError::NoSheetsFound));
        }
    };

    // 1. 读取表头行
    let mut first_row: Vec<String> = Vec::new();
    // 逐个单元格处理，直到找到第一行的所有单元格
    while let Some(cell) = cell_reader.next_cell()? {
        if cell.get_position().0 > 0 {
            // 已经读取完第一行
            break;
        }
        // 安全地转换单元格值为字符串
        match cell.get_value().as_string() {
            Some(value) => first_row.push(value),
            None => {
                return Err(Box::new(ExcelError::CellConversionFailed(format!(
                    "无法将单元格 {:?} 转换为字符串",
                    cell.get_position()
                ))))
            }
        }
    }
    if first_row.is_empty() {
        log::warn!("工作表为空或没有数据");
        return Err(Box::new(ExcelError::InvalidFirstLine));
    }
    log::debug!("header_cells: {:?}", first_row);

    // 查找标签索引
    let tag_index = find_tag_index(&first_row, &parsed_cfg.tag_name)?;

    // 查找语言索引
    let lang_index_map = find_language_indices(&first_row, &parsed_cfg.lang_map);
    parsed_cfg.tag_index = tag_index;
    parsed_cfg.lang_index_map = lang_index_map;
    Ok(parsed_cfg)
}

/// 打开Excel工作簿
pub fn open_excel_workbook(file_path: &str) -> Result<Xlsx<BufReader<File>>, Box<dyn Error>> {
    let workbook: Xlsx<_> = open_workbook(file_path)?;
    Ok(workbook)
}

/// 查找标签索引
fn find_tag_index(first_row: &[String], tag_name: &str) -> Result<u32, Box<dyn Error>> {
    first_row
        .iter()
        .position(|r| r == tag_name)
        .map(|pos| pos as u32)
        .ok_or_else(|| Box::new(ExcelError::TagNotFound(tag_name.to_string())) as Box<dyn Error>)
}

/// 查找语言索引
fn find_language_indices(
    first_row: &[String],
    lang_map: &[(String, String)],
) -> Vec<(String, u32)> {
    lang_map
        .iter()
        .filter_map(|(lang, lang_name)| {
            first_row
                .iter()
                .position(|r| r == lang_name)
                .map(|index| (lang.clone(), index as u32))
        })
        .collect()
}

/// 一次解析单个语种
/// 内存占用低，解析全部语言更耗时
/// * @param workbook Excel工作簿
/// * @param sheet_name 工作表名称
/// * @param tag_index 标签列索引
/// * @param lang_index 语言列索引
/// * @param tag_value_map 标签值映射
/// * @return 解析是否成功
pub fn process_excel_single_lang(
    workbook: &mut Xlsx<BufReader<File>>,
    sheet_name: &str,
    tag_index: u32,
    lang_index: u32,
    tag_value_map: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cell_reader = workbook.worksheet_cells_reader(sheet_name)?;
    let mut cur = RowSingleLangData::default();

    while let Some(cell) = cell_reader.next_cell()? {
        // 其余代码保持不变
        let (row, col) = cell.get_position();

        // 换行时处理上一行数据
        if let Some(prev_row) = cur.row {
            if row != prev_row {
                if let Some(tag) = &cur.tag {
                    let tag_trim = tag.trim();
                    if !tag_trim.is_empty() {
                        tag_value_map
                            .insert(tag_trim.to_string(), cur.value.clone().unwrap_or_default());
                    }
                }
                cur = RowSingleLangData::default();
            }
        }

        cur.row = Some(row);
        if row == 0 || (col != tag_index && col != lang_index) {
            // 跳过表头行和非相关列
            continue;
        }

        if col == tag_index {
            cur.tag = cell.get_value().as_string();
        } else if col == lang_index {
            let raw = cell.get_value().as_string().unwrap_or("".to_owned());
            cur.value = Some(raw);
        }
    }

    // 处理最后一行数据
    if let Some(tag) = &cur.tag {
        let tag_trim = tag.trim();
        if !tag_trim.is_empty() {
            tag_value_map.insert(tag_trim.to_string(), cur.value.unwrap_or_default());
        }
    }

    Ok(())
}

/// 一次解析所有语种
/// * 解析全部语言耗时更少，但内存占用更高
/// * @param workbook Excel工作簿
/// * @param sheet_name 工作表名称
/// * @param tag_index 标签列索引
/// * @param lang_index_vec 语言列索引向量
/// * @param translation_rows 按语言槽位存储的翻译行
/// * @return 解析是否成功
pub fn process_excel_multi_lang(
    workbook: &mut Xlsx<BufReader<File>>,
    sheet_name: &str,
    tag_index: u32,
    lang_index_vec: &[u32],
    translation_rows: &mut Vec<TranslationRow>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cell_reader = workbook.worksheet_cells_reader(sheet_name)?;
    let max_column = lang_index_vec
        .iter()
        .copied()
        .chain(std::iter::once(tag_index))
        .max()
        .unwrap_or(tag_index) as usize;
    let mut language_slot_by_column = vec![None; max_column + 1];
    for (slot, column) in lang_index_vec.iter().copied().enumerate() {
        language_slot_by_column[column as usize] = Some(slot);
    }
    let mut cur = RowMultiLangData::new(lang_index_vec.len());

    while let Some(cell) = cell_reader.next_cell()? {
        // 其余代码保持不变
        let (row, col) = cell.get_position();

        // 换行时处理上一行数据
        if let Some(prev_row) = cur.row {
            if row != prev_row {
                push_translation_row(&mut cur, translation_rows);
                cur = RowMultiLangData::new(lang_index_vec.len());
            }
        }

        cur.row = Some(row);
        if row == 0 {
            // 跳过表头行和非相关列
            continue;
        }

        if col == tag_index {
            // 处理tag
            cur.tag = cell.get_value().as_string();
        } else if let Some(Some(slot)) = language_slot_by_column.get(col as usize) {
            let raw = cell.get_value().as_string().unwrap_or("".to_owned());
            cur.values[*slot] = raw;
        }
    }

    // 处理最后一行数据
    push_translation_row(&mut cur, translation_rows);

    Ok(())
}
