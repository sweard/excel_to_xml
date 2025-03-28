use std::{
    collections::{HashMap, HashSet},
    fs::{remove_file, rename, File},
    io::{BufReader, BufWriter},
};

use crate::config::ParsedCfg;
use calamine::{open_workbook, Data, DataType, Reader, Xlsx};
use quick_xml::{
    events::{BytesText, Event},
    Writer,
};

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
    // // 打开Excel文件
    // let workbook: &mut Xlsx<_> = &mut open_workbook(file_path)?;

    // // 获取sheet名称
    // let sheet_name = if parsed_cfg.sheet_name.is_empty() {
    //     workbook
    //         .sheet_names()
    //         .get(0)
    //         .cloned()
    //         .ok_or("No sheets found")?
    // } else {
    //     parsed_cfg.sheet_name.clone()
    // };

    // let range: calamine::Range<calamine::Data> = workbook.worksheet_range(&sheet_name)?;
    let tag_index = parsed_cfg.tag_index;

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

        // 构建标签值映射 - 优化：预分配容量
        let lang_index = *index;
        // let tag_value_map = process_excel_data(&range, tag_index as usize, lang_index as usize)?;

        let tag_value_map = process_excel_data2(file_path, parsed_cfg, tag_index, lang_index)?;

        // 处理XML文件
        update_xml_file(path, &tag_value_map, parsed_cfg)?;

        drop(tag_value_map); 
    }

    Ok(())
}

// 从Excel提取数据到映射表
fn process_excel_data(
    range: &calamine::Range<Data>,
    tag_index: usize,
    lang_index: usize,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    // 根据Excel行数预分配合适的容量
    let estimated_capacity = range.rows().count().saturating_sub(1);
    let mut tag_value_map = HashMap::with_capacity(estimated_capacity);
    for row in range.rows().skip(1) {
        if let (Some(tag), Some(value)) = (row.get(tag_index), row.get(lang_index)) {
            if !tag.is_empty() {
                tag_value_map.insert(
                    tag.to_string().trim().to_string(),
                    value.to_string().replace('\n', "\\n"),
                );
            }
        }
    }
    Ok(tag_value_map)
}

fn process_excel_data2(
    file_path: &str,
    parsed_cfg: &ParsedCfg,
    tag_index: i32,
    lang_index: i32,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    // 根据Excel行数预分配合适的容量
    let mut tag_value_map = HashMap::with_capacity(8000);
    // 打开Excel文件
    let workbook: &mut Xlsx<_> = &mut open_workbook(file_path)?;

    // 获取sheet名称
    let sheet_name = if parsed_cfg.sheet_name.is_empty() {
        workbook
            .sheet_names()
            .get(0)
            .cloned()
            .ok_or("No sheets found")?
    } else {
        parsed_cfg.sheet_name.clone()
    };
    let mut cell_reader = workbook.worksheet_cells_reader(&sheet_name)?;
    let mut cur: (Option<u32>, Option<String>, Option<String>) = (None, None, None);
    while let Some(cell) = cell_reader.next_cell()? {
        let (row, col) = cell.get_position();
        let i32_col = col as i32;
         // 换行时重置cur,并将上一行的数据插入到map中
         if let Some(_row) = cur.0 {
            if row != _row {
                if let (_,Some(tag), Some(value)) = cur {
                    if !tag.is_empty() {
                        tag_value_map.insert(
                            tag.to_string().trim().to_string(),
                            value.to_string().replace('\n', "\\n"),
                        );
                    }
                }
                cur = (None, None, None);
            }
        }

        cur.0 = Some(row);
        if row == 0 || (i32_col != tag_index && i32_col != lang_index) {
            // 跳过第一行以及非标签和语言列
            continue;
        }
        if i32_col == tag_index {
            cur.1 = cell.get_value().as_string();
        } else if i32_col == lang_index {
            cur.2 = Some(cell.get_value().as_string().unwrap_or("".to_owned()));
        }
    }
    Ok(tag_value_map)
}

// 更新XML文件
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
        // TODO 优化：直接替换整个文件
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
                // 提前处理没有当前标签的情况，减少嵌套
                if current_tag_name.is_none() {
                    xml_writer.write_event(Event::Text(e.to_owned()))?;
                    continue;
                }

                let tag_name = current_tag_name.as_ref().unwrap();
                updated_tags.insert(tag_name.to_string());

                // 使用match替代嵌套的if-else，提高可读性
                match tag_value_map.get(tag_name) {
                    Some(value) if !value.is_empty() => {
                        // 有映射值且不为空，使用新值
                        xml_writer.write_event(Event::Text(BytesText::new(value)))?;
                    }
                    _ => {
                        // 没有映射值或值为空，保持原值
                        xml_writer.write_event(Event::Text(e.to_owned()))?;
                    }
                }

                // 处理完后清除当前标签
                current_tag_name = None;
            }

            Ok(Event::Eof) => break,

            Ok(e) => xml_writer.write_event(e)?,

            Err(e) => return Err(e.into()),
        }
        buf.clear();
    }

    // 替换原文件
    drop(xml_writer); // 确保文件被写入
    remove_file(path)?;
    rename(temp_path, path)?;

    Ok(())
}

// 添加缺失的标签
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
            let mut elem = quick_xml::events::BytesStart::new("string");
            elem.push_attribute(("name", tag.as_str()));

            xml_writer.write_event(Event::Start(elem))?;
            xml_writer.write_event(Event::Text(BytesText::new(value)))?;
            xml_writer.write_event(Event::End(quick_xml::events::BytesEnd::new("string")))?;
        }
    }

    // 添加最后的换行
    xml_writer.write_event(Event::Text(BytesText::new("\n")))?;

    Ok(())
}
