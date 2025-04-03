use std::{
    fs,
    io::{self, Write},
};
mod config;
mod find_files;
mod read_excel;
mod write_xml;

fn main() {
    // // 读取配置文件json
    // let mut cfg_json: Option<String> = None;
    // // 读取Excel路径
    // let mut excel_path = String::new();
    // // 读取XML所在模块路径
    // let mut xml_dir_path = String::new();
     // 读取配置文件json
     let mut cfg_json: Option<String> = Some("/Users/jeff/RustProjects/parseExcel/cfg_json".to_string());
     // 读取Excel路径
     let mut excel_path = String::from("/Users/jeff/RustProjects/parseExcel/APP文案翻译汇总表2.1.xlsx");
     // 读取XML所在模块路径
     let mut xml_dir_path = String::from("/Users/jeff/StudioProjects/switchbot-rn/android/switchbot-common");
    let menu = "c:更新配置文件路径\ne:更新xlsx路径\nx:更新xml所在文件夹路径\nu:同步\nqu:快速同步（内存占用多一点）\ni:查看当前配置信息\nm:菜单\nq:退出";
    let json_prompt = "请输入配置文件路径:";
    let excel_prompt = "请输入Excel路径:";
    let xml_prompt = "请输入XML所在模块路径:";
    println!("欢迎使用xml更新工具");
    println!("{}", menu);
    loop {
        let input = prompt_user_input("");
        match input.as_str() {
            "c" => {
                cfg_json = update_cfg_json(json_prompt);
            }
            "e" => {
                excel_path = prompt_user_input(excel_prompt);
            }
            "x" => {
                xml_dir_path = prompt_user_input(xml_prompt);
            }
            "u" => {
                if let Some(cfg) = &cfg_json {
                    // 统计耗时
                    let start_time = std::time::Instant::now();
                    match write_xml::update(cfg, &excel_path, &xml_dir_path) {
                        Ok(_) => println!("更新成功"),
                        Err(e) => println!("更新失败: {:?}", e),
                    }
                    let duration = start_time.elapsed();
                    println!("同步耗时: {:?}", duration);
                } else {
                    println!("配置文件路径无效");
                }
            }
            "qu" => {
                if let Some(cfg) = &cfg_json {
                    // 统计耗时
                    let start_time = std::time::Instant::now();
                    match write_xml::quick_update(cfg, &excel_path, &xml_dir_path) {
                        Ok(_) => println!("更新成功"),
                        Err(e) => println!("更新失败: {:?}", e),
                    }
                    let duration = start_time.elapsed();
                    println!("快速同步耗时: {:?}", duration);
                } else {
                    println!("配置文件路径无效");
                }
            }
            "i" => {
                if let Some(cfg) = &cfg_json {
                    println!(
                        "当前配置 json:{} \nexcel:{} \nxml dir:{}",
                        &cfg, &excel_path, &xml_dir_path
                    );
                } else {
                    println!(
                        "当前配置 json:{} \nexcel:{} \nxml dir:{}",
                        "格式异常", &excel_path, &xml_dir_path
                    );
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
    if !prompt.is_empty() {
        println!("{}", prompt);
    }
    io::stdout().flush().expect("无法刷新标准输出");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("读取输入时出错");
    input.trim().replace("'", "")
}

fn update_cfg_json(prompt: &str) -> Option<String> {
    let cfg_json_path = prompt_user_input(prompt);
    match fs::read_to_string(&cfg_json_path) {
        Ok(content) => Some(content),
        Err(e) => {
            println!("读取配置文件时出错: {:?}", e);
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn write() {
        // 读取配置文件json
        let mut cfg_json: Option<String> = Some("/Users/jeff/RustProjects/parseExcel/cfg_json".to_string());
        // 读取Excel路径
        let mut excel_path = String::from("/Users/jeff/RustProjects/parseExcel/APP文案翻译汇总表2.1.xlsx");
        // 读取XML所在模块路径
        let mut xml_dir_path = String::from("/Users/jeff/StudioProjects/switchbot-rn/android/switchbot-common");
        let menu = "c:更新配置文件路径\nx:更新xlsx路径\nt:更新xml所在文件夹路径\nu:同步\nqu:快速同步（内存占用多一点）\ni:查看当前配置信息\nq:退出";
        let json_prompt = "请输入配置文件路径:";
        let excel_prompt = "请输入Excel路径:";
        let xml_prompt = "请输入XML所在模块路径:";
        loop {
            let input = prompt_user_input(menu);
            match input.as_str() {
                "c" => {
                    cfg_json = update_cfg_json(json_prompt);
                }
                "x" => {
                    excel_path = prompt_user_input(excel_prompt);
                }
                "t" => {
                    xml_dir_path = prompt_user_input(xml_prompt);
                }
                "u" => {
                    if let Some(cfg) = &cfg_json {
                        // 统计耗时
                        let start_time = std::time::Instant::now();
                        match write_xml::update(cfg, &excel_path, &xml_dir_path) {
                            Ok(_) => println!("更新成功"),
                            Err(e) => println!("更新失败: {:?}", e),
                        }
                        let duration = start_time.elapsed();
                        println!("同步耗时: {:?}", duration);
                    } else {
                        println!("配置文件路径无效");
                    }
                }
                "qu" => {
                    if let Some(cfg) = &cfg_json {
                        // 统计耗时
                        let start_time = std::time::Instant::now();
                        match write_xml::quick_update(cfg, &excel_path, &xml_dir_path) {
                            Ok(_) => println!("更新成功"),
                            Err(e) => println!("更新失败: {:?}", e),
                        }
                        let duration = start_time.elapsed();
                        println!("快速同步耗时: {:?}", duration);
                    } else {
                        println!("配置文件路径无效");
                    }
                }
                "i" => {
                    if let Some(cfg) = &cfg_json {
                        println!(
                            "当前配置 json:{} \nexcel:{} \nxml dir:{}",
                            &cfg, &excel_path, &xml_dir_path
                        );
                    } else {
                        println!(
                            "当前配置 json:{} \nexcel:{} \nxml dir:{}",
                            "格式异常", &excel_path, &xml_dir_path
                        );
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
}
