
use crate::config::{InputCfg, ParsedCfg};
use calamine::{open_workbook, Reader, Xlsx};

/**
 * 解析Excel文件
 * @param file_path excel文件路径
 * @param config_json 用户输入的配置JSON
 * @return 解析后的配置
 */
pub fn parse_cfc_with_excel(
    file_path: &str,
    config_json: &str,
) -> Result<ParsedCfg, Box<dyn std::error::Error>> {
    // 解析配置JSON
    let input_cfg: InputCfg = InputCfg::from_json(config_json)?;

    // 打开Excel文件
    let mut workbook: Xlsx<_> = open_workbook(file_path)?;
    let sheet_name = workbook
        .sheet_names()
        .get(0)
        .cloned()
        .ok_or("No sheets found")?;
    let range = workbook.worksheet_range(&sheet_name)?;

    // 获取第一行内容
    let first_row: Vec<String> = range
        .rows()
        .next()
        .ok_or("The sheet is empty")?
        .iter()
        .map(|cell| cell.to_string())
        .collect();

    // 查找标签索引
    let tag_index = first_row
        .iter()
        .position(|r| r == &input_cfg.tag_name)
        .ok_or("Tag name not found in the first row")? as i32;

    // 查找语言索引
    let mut lang_index_map = Vec::new();
    for (lang, lang_name) in &input_cfg.lang_map {
        if let Some(index) = first_row.iter().position(|r| r == lang_name) {
            lang_index_map.push((lang.clone(), index as i32));
        }
    }

    // 构建ParsedCfg
    let parsed_cfg = ParsedCfg {
        sheet_name: input_cfg.sheet_name,
        tag_index,
        default_lang: input_cfg.default_lang,
        lang_index_map,
        replace_all: input_cfg.replace_all,
    };

    Ok(parsed_cfg)
}
