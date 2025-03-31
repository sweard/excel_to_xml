use std::{
    collections::{HashMap, HashSet},
    fs::{remove_file, rename, File},
    io::{BufReader, BufWriter},
};

use crate::config::ParsedCfg;
use calamine::{open_workbook, DataType, Reader, Xlsx};
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Writer,
};

/// 当前行数据结构体，替代元组提高可读性
#[derive(Default)]
struct RowData {
    row: Option<u32>,
    tag: Option<String>,
    value: Option<String>,
}

/**
 * 写入XML文件
 * @param file_path excel文件路径
 * @param parsed_cfg 解析后的配置
 * @param paths XML文件路径
 */
pub fn write_xml(
    file_path: &str,
    parsed_cfg: &ParsedCfg,
    paths: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let tag_index = parsed_cfg.tag_index;
    
    // 预先打开Excel文件，只打开一次
    let mut workbook: Xlsx<_> = open_workbook(file_path)?;
    
    // 获取sheet名称（只需获取一次）
    let sheet_name = if parsed_cfg.sheet_name.is_empty() {
        workbook
            .sheet_names()
            .get(0)
            .cloned()
            .ok_or("No sheets found")?
    } else {
        parsed_cfg.sheet_name.clone()
    };

    // 预分配一个HashMap，循环内复用
    let mut tag_value_map = HashMap::with_capacity(5000);

    // 遍历语言索引映射
    for (lang, index) in &parsed_cfg.lang_index_map {
        let is_default_lang = lang == &parsed_cfg.default_lang;
        let end_point = if is_default_lang {
            "values/strings.xml".to_string()
        } else {
            format!("values-{}/strings.xml", lang)
        };

        // 查找匹配的XML文件路径
        let path = match paths.iter().find(|path| path.ends_with(&end_point)) {
            Some(path) => path,
            None => continue, // 没找到对应语言的文件，跳过这个语言
        };

        // 清空map，准备复用
        tag_value_map.clear();
        
        // 构建标签值映射 - 复用workbook、sheet_name和tag_value_map
        let lang_index = *index;
        process_excel_data(&mut workbook, &sheet_name, tag_index, lang_index, &mut tag_value_map)?;

        // 处理XML文件
        update_xml_file(path, &tag_value_map, parsed_cfg)?;
    }
    tag_value_map.clear(); // 清空map
    tag_value_map.shrink_to_fit(); // 释放内存
    Ok(())
}

/// 从Excel提取数据到映射表 - 使用传入的workbook和tag_value_map避免重复创建
fn process_excel_data(
    workbook: &mut Xlsx<BufReader<File>>,
    sheet_name: &str,
    tag_index: i32,
    lang_index: i32,
    tag_value_map: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cell_reader = workbook.worksheet_cells_reader(sheet_name)?;
    let mut cur = RowData::default();
    
    while let Some(cell) = cell_reader.next_cell()? {
        // 其余代码保持不变
        let (row, col) = cell.get_position();
        let i32_col = col as i32;
        
        // 换行时处理上一行数据
        if let Some(prev_row) = cur.row {
            if row != prev_row {
                if let (Some(tag), Some(value)) = (&cur.tag, &cur.value) {
                    let tag_trim = tag.trim();
                    if !tag_trim.is_empty() {
                        tag_value_map.insert(
                            tag_trim.to_string(),
                            value.replace('\n', "\\n"),
                        );
                    }
                }
                cur = RowData::default();
            }
        }

        cur.row = Some(row);
        if row == 0 || (i32_col != tag_index && i32_col != lang_index) {
            // 跳过表头行和非相关列
            continue;
        }
        
        if i32_col == tag_index {
            cur.tag = cell.get_value().as_string().map(String::from);
        } else if i32_col == lang_index {
            cur.value = Some(cell.get_value().as_string().unwrap_or("".to_owned()));
        }
    }
    
    // 处理最后一行数据
    if let (Some(tag), Some(value)) = (&cur.tag, &cur.value) {
        let tag_trim = tag.trim();
        if !tag_trim.is_empty() {
            tag_value_map.insert(
                tag_trim.to_string(),
                value.replace('\n', "\\n"),
            );
        }
    }
    
    Ok(())
}

/// 更新XML文件
fn update_xml_file(
    path: &str,
    tag_value_map: &HashMap<String, String>,
    parsed_cfg: &ParsedCfg,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建临时文件路径
    let temp_path = format!("{}.temp", path);

    // 打开原始XML文件和临时文件
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut xml_reader = quick_xml::Reader::from_reader(reader);

    let output_file = File::create(&temp_path)?;
    let writer = BufWriter::new(output_file);
    let mut xml_writer = Writer::new(writer);

    let mut buf = Vec::new();
    let mut current_tag_name = None;
    let mut updated_tags = HashSet::new();
    if parsed_cfg.replace_all {
        // TODO 覆盖全部内容
    }

    // 处理XML事件
    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"string" {
                    // 提取name属性
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            if attr.key.as_ref() == b"name" {
                                if let Ok(tag_name) = std::str::from_utf8(attr.value.as_ref()) {
                                    if tag_value_map.contains_key(tag_name) {
                                        current_tag_name = Some(tag_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                xml_writer.write_event(Event::Start(e.to_owned()))?;
            }

            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"resources" {
                    // 在关闭resources标签前添加缺失的标签
                    add_missing_tags(&mut xml_writer, tag_value_map, &updated_tags)?;
                }
                xml_writer.write_event(Event::End(e.to_owned()))?;
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
                    Some(value) if !value.is_empty() => {
                        xml_writer.write_event(Event::Text(BytesText::new(value)))?;
                    }
                    _ => {
                        xml_writer.write_event(Event::Text(e.to_owned()))?;
                    }
                }

                current_tag_name = None;
            }

            Ok(Event::Eof) => break,
            Ok(e) => xml_writer.write_event(e)?,
            Err(e) => return Err(e.into()),
        }
        buf.clear();
    }

    // 替换原文件
    drop(xml_writer); // 确保文件被写入并关闭
    remove_file(path)?;
    rename(temp_path, path)?;

    Ok(())
}

/// 添加缺失的标签
fn add_missing_tags(
    xml_writer: &mut Writer<BufWriter<File>>,
    tag_value_map: &HashMap<String, String>,
    updated_tags: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for (tag, value) in tag_value_map {
        if !updated_tags.contains(tag) {
            // 为每个新标签添加缩进
            xml_writer.write_event(Event::Text(BytesText::new("\n    ")))?;

            // 创建新标签
            let mut elem = BytesStart::new("string");
            elem.push_attribute(("name", tag.as_str()));

            xml_writer.write_event(Event::Start(elem))?;
            xml_writer.write_event(Event::Text(BytesText::new(value)))?;
            xml_writer.write_event(Event::End(BytesEnd::new("string")))?;
        }
    }

    // 添加最后的换行
    xml_writer.write_event(Event::Text(BytesText::new("\n")))?;

    Ok(())
}
