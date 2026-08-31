use calamine::Reader;

mod config;
mod find_files;
mod read_excel;
mod write_xml;

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

/// 更新XML文件
pub fn update(cfg_json: &str, excel_path: &str, xml_dir_path: &str) -> Result<(), String> {
    write_xml::update(cfg_json, excel_path, xml_dir_path).map_err(|err| format!("更新失败: {err}"))
}

/// 快速更新XML文件
pub fn quick_update(cfg_json: &str, excel_path: &str, xml_dir_path: &str) -> Result<(), String> {
    write_xml::quick_update(cfg_json, excel_path, xml_dir_path)
        .map_err(|err| format!("快速更新失败: {err}"))
}
