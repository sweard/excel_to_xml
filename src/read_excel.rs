use crate::config::{InputCfg, ParsedCfg};
use calamine::{open_workbook, DataType, Reader, Xlsx};
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

    // 获取cell_reader迭代器
    let mut cell_reader = workbook.worksheet_cells_reader(&sheet_name)?;
    
     // 1. 读取表头行
     let mut first_row:Vec<String> = Vec::new();
     // 逐个单元格处理，直到找到第一行的所有单元格
     while let Some(cell) = cell_reader.next_cell()? {
         if cell.get_position().0 > 0 {
             // 已经读取完第一行
             break;
         }
          // 安全地转换单元格值为字符串
        match cell.get_value().as_string() {
            Some(value) => first_row.push(value),
            None => return Err(Box::new(ExcelError::CellConversionFailed(
                format!("无法将单元格 {:?} 转换为字符串", cell.get_position())
            ))),
        }
     }
     if first_row.is_empty() {
         return Err(Box::new(ExcelError::InvalidFirstLine));
     }
     println!("header_cells: {:?}", first_row);

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
