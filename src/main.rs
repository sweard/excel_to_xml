use std::{fs, io::{self, Write}, mem};
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
    let cfg_json = loop {
        let cfg_json_path = prompt_user_input("请输入配置文件路径:");
        match fs::read_to_string(&cfg_json_path) {
            Ok(content) => {
                break content;
            }
            Err(e) => {
                println!("读取配置文件时出错: {:?}", e);
            }
        }; 
    };

    let (excel_path,parsed_cfg) = loop {
        let excel_path = prompt_user_input("请输入Excel路径:");
        match read_excel::parse_cfg_with_excel(&excel_path, cfg_json.as_str()) {
            Ok(parsed_cfg) => {
                println!("解析配置成功: {:?}", parsed_cfg);
                break (excel_path,parsed_cfg);
            }
            Err(e) => {
                eprintln!("解析配置时出错: {:?}", e);
            }
        }
    };
    println!("Size of parsed_cfg: {}", calculate_vec_memory(&parsed_cfg.lang_index_map));
    let paths = loop {
        let xml_dir_path = prompt_user_input("请输入XML所在模块路径:");
        if let Some(res_folder) = find_files::find_target_folder(&xml_dir_path, "res", &FILTER) {
            println!("找到res文件夹: {}", res_folder);
            break find_files::collect_target_files(&res_folder, "values", "strings.xml");
        } else {
            println!("未找到res文件夹");
        }
    };

    loop {
        let input = prompt_user_input("r更新xml文件，q退出:");
        match input.as_str() {
            "r" => {
                if let Err(e) = write_xml::write_xml(&excel_path, &parsed_cfg, &paths) {
                    println!("写入XML时出错: {:?}", e);
                } else {
                    println!("写入XML成功");
                }
            }
            "q" => {
                break;
            }
            _ => {
                println!("无效输入");
            }
            
        }
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

fn calculate_vec_memory<T>(vec: &Vec<T>) -> usize {
    let element_size = mem::size_of::<T>();
    let heap_size = element_size * vec.len();
    let stack_size = mem::size_of_val(vec);
    stack_size + heap_size
}

