use std::{fs, io::{self, Write}};
mod find_files;
mod read_excel;
mod write_xml;
mod config;

const FILTER: [&str; 2] = ["build", "mainland"];
// const CFG_JSON: &str = r#"
// {
//     "sheetName":"",
//     "tagName": "Android tag",
//     "defaultLang":"en",
//     "langMap": {
//         "zh": "中文简体",
//         "zh-rTW": "中文繁体",
//         "en": "英语",
//         "ja": "日语",
//         "ko-rKR": "韩语",
//         "fr": "法语",
//         "de": "德语",
//         "es": "西班牙语",
//         "it": "意大利语",
//         "nl": "荷兰语"
//     },
//     "replaceAll": false
// }
// "#;

fn main() {
    let cfg_json_path = prompt_user_input("请输入配置文件路径:");
    let cfg_json = match fs::read_to_string(&cfg_json_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("读取配置文件时出错: {:?}", e);
            return;
        }
    };

    let xml_dir_path = prompt_user_input("请输入XML所在模块路径:");
    let excel_path = prompt_user_input("请输入Excel路径:");

    if let Some(res_folder) = find_files::find_target_folder(&xml_dir_path, "res", &FILTER) {
        println!("找到res文件夹: {}", res_folder);
        let paths = find_files::collect_target_files(&res_folder, "values", "strings.xml");

        match read_excel::parse_cfg_with_excel(&excel_path, cfg_json.as_str()) {
            Ok(parsed_cfg) => {
                println!("解析配置成功: {:?}", parsed_cfg);
                if let Err(e) = write_xml::write_xml(&excel_path, &parsed_cfg, &paths) {
                    eprintln!("写入XML时出错: {:?}", e);
                }
            }
            Err(e) => {
                eprintln!("解析配置时出错: {:?}", e);
            }
        }
    } else {
        println!("未找到res文件夹");
    }
}

/// 提示用户输入并返回去除多余字符的字符串
fn prompt_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    io::stdout().flush().expect("无法刷新标准输出");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("读取输入时出错");
    input.trim().replace("'", "")
}

