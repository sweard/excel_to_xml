use crate::config::{InputCfg, ParsedCfg};
use calamine::{open_workbook, Data, Reader, Xlsx};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;

/// 自定义错误类型，用于更清晰的错误处理
#[derive(Debug)]
pub enum ExcelError {
    NoSheetsFound,
    EmptySheet,
    TagNotFound(String),
    InvalidExcelFormat(String),
}

impl fmt::Display for ExcelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExcelError::NoSheetsFound => write!(f, "未找到任何工作表"),
            ExcelError::EmptySheet => write!(f, "工作表为空"),
            ExcelError::TagNotFound(tag) => write!(f, "未找到标签: {}", tag),
            ExcelError::InvalidExcelFormat(msg) => write!(f, "Excel 格式错误: {}", msg),
        }
    }
}

impl Error for ExcelError {}

/**
 * 解析Excel文件
 * @param file_path Excel文件路径
 * @param config_json 用户输入的配置JSON
 * @return 解析后的配置
 */
pub fn parse_cfg_with_excel(
    file_path: &str,
    config_json: &str,
) -> Result<ParsedCfg, Box<dyn Error>> {
    // 解析配置JSON
    let input_cfg = InputCfg::from_json(config_json)?;

    // 打开Excel文件并获取工作表
    let (mut workbook, sheet_name) = open_excel_workbook(file_path)?;

    // 获取工作表范围
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| ExcelError::InvalidExcelFormat(e.to_string()))?;

    // 解析表头行
    let first_row = parse_header_row(&range)?;

    // 查找标签索引
    let tag_index = find_tag_index(&first_row, &input_cfg.tag_name)?;

    // 查找语言索引
    let lang_index_map = find_language_indices(&first_row, &input_cfg.lang_map.as_slice());

    // 构建解析后的配置
    Ok(ParsedCfg {
        sheet_name: input_cfg.sheet_name,
        tag_index,
        default_lang: input_cfg.default_lang,
        lang_index_map,
        replace_all: input_cfg.replace_all,
    })
}

/// 打开Excel文件并获取第一个工作表名称
fn open_excel_workbook(file_path: &str) -> Result<(Xlsx<BufReader<File>>, String), Box<dyn Error>> {
    let workbook: Xlsx<_> = open_workbook(file_path)?;
    let sheet_name = workbook
        .sheet_names()
        .get(0)
        .cloned()
        .ok_or(ExcelError::NoSheetsFound)?;
    Ok((workbook, sheet_name))
}

/// 解析表头行
fn parse_header_row(range: &calamine::Range<Data>) -> Result<Vec<String>, Box<dyn Error>> {
    let first_row: Vec<String> = range
        .rows()
        .next()
        .ok_or(ExcelError::EmptySheet)?
        .iter()
        .map(|cell| cell.to_string())
        .collect();
    Ok(first_row)
}

/// 查找标签索引
fn find_tag_index(first_row: &[String], tag_name: &str) -> Result<i32, Box<dyn Error>> {
    first_row
        .iter()
        .position(|r| r == tag_name)
        .map(|pos| pos as i32)
        .ok_or_else(|| Box::new(ExcelError::TagNotFound(tag_name.to_string())) as Box<dyn Error>)
}

/// 查找语言索引
fn find_language_indices(
    first_row: &[String],
    lang_map: &[(String, String)],
) -> Vec<(String, i32)> {
    lang_map
        .iter()
        .filter_map(|(lang, lang_name)| {
            first_row
                .iter()
                .position(|r| r == lang_name)
                .map(|index| (lang.clone(), index as i32))
        })
        .collect()
}
