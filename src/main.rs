use std::io::stdin;
mod find_files;
mod read_excel;
mod write_xml;
mod config;


fn main() {
    println!("请输入XML所在模块路径:");
    let mut xml_dir = String::new();
    stdin().read_line(&mut xml_dir).expect("error");
    let xml_dir_path = xml_dir.trim().replace("'", "");
    let mut filter: Vec<&str> = Vec::new(); 
    filter.push("build");
    filter.push("mainland");
    let res = find_files::find_target_folder(&xml_dir_path, "res", &filter);

    println!("请输入Excel路径:");
    let mut excel_dir = String::new();
    stdin().read_line(&mut excel_dir).expect("error");

    match res {
        Some(value) => {
            println!("find res folder:{}", value);
            let paths = find_files::collect_target_files(&value, "values", "strings.xml");
            let excel_path = excel_dir.trim().replace("'", "");
            let cfg_json = r#"
        {
            "sheetName":"",
            "tagName": "Android tag",
            "defaultLang":"en",
            "langMap": {
                "zh": "中文简体",
                "zh-rTW": "中文繁体",
                "en": "英语",
                "ja": "日语",
                "ko-rKR": "韩语",
                "fr": "法语",
                "de": "德语",
                "es": "西班牙语",
                "it": "意大利语",
                "nl": "荷兰语"
            },
            "replaceAll": false
        }
        "#;
            let parsed_cfg = read_excel::parse_cfc_with_excel(&excel_path, cfg_json);
            match parsed_cfg {
                Ok(value) => {
                    println!("parsed_cfg-->{:?}", value);
                    write_xml::write_xml(&excel_path,&value, &paths).expect("write_xml error");
                }
                Err(e) => {
                    println!("error-->{:?}", e);
                }
                
            }
        }
        None => {
            println!("not find res folder");
        }
    }
}

