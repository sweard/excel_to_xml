use calamine::Reader;

mod config;
mod find_files;
mod read_excel;
mod write_xml;

pub use write_xml::{ConversionMode, ConversionReport};

/// 获取Excel文件中的工作表名称列表
pub fn get_sheet_names(file_path: &str) -> Result<Vec<String>, String> {
    read_excel::open_excel_workbook(file_path)
        .map(|workbook| workbook.sheet_names())
        .map_err(|err| format!("读取 Excel 工作表失败: {err}"))
}

/// 获取默认配置JSON字符串
pub fn get_default_cfg_json() -> String {
    config::CFG_JSON.to_string()
}

/// 将Excel翻译内容转换到Android XML资源文件。
pub fn convert(
    cfg_json: &str,
    excel_path: &str,
    xml_dir_path: &str,
    mode: ConversionMode,
) -> Result<ConversionReport, String> {
    write_xml::convert(cfg_json, excel_path, xml_dir_path, mode)
        .map_err(|err| format!("转换失败: {err}"))
}
