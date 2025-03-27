use serde_json::{from_str, Value};
use std::error::Error;

/**
 * 输入的配置
 */
#[derive(Debug, PartialEq)]
pub struct InputCfg {
    pub sheet_name: String,              // 表名
    pub tag_name: String,                // 标签名称
    pub default_lang: String,            // 默认语言
    pub lang_map: Vec<(String, String)>, // 语言名称 zh-cn,中文
    pub replace_all: bool,               // 是否替换所有
}

impl InputCfg {
    /**
     * {"tagName":"Android tag",
     * "langMap":{"zh":"中文简体","zh-rTW":"中文繁体","en":"英语","ja":"日语","ko-rKR":"韩语","fr":"法语","de":"德语","es":"西班牙语","it":"意大利语","nl":"荷兰语"},
     * "replaceAll":false}
     */
    pub fn from_json(json: &str) -> Result<Self, Box<dyn Error>> {
        let parsed_json: Value = from_str(json)?;
        let json_obj = parsed_json.as_object().ok_or("Invalid JSON format")?;

        let sheet_name = json_obj
            .get("sheetName")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let tag_name = json_obj
            .get("tagName")
            .and_then(Value::as_str)
            .ok_or("Missing or invalid 'tagName' field")?
            .to_string();

        let default_lang = json_obj
            .get("defaultLang")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let lang_map = json_obj
            .get("langMap")
            .and_then(Value::as_object)
            .ok_or("Missing or invalid 'langMap' field")?
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect();
        // 是否替换所有,默认为false
        let replace_all = json_obj
            .get("replaceAll")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        Ok(InputCfg {
            sheet_name,
            tag_name,
            default_lang,
            lang_map,
            replace_all,

        })
    }
}

/**
 * 解析完excel后生成的配置
 */
#[derive(Debug, PartialEq)]
pub struct ParsedCfg {
    pub sheet_name: String,                 // 表名
    pub tag_index: i32,                     // 标签序号 excel中的序号
    pub default_lang: String,               // 默认语言
    pub lang_index_map: Vec<(String, i32)>, // 语言名称 zh-cn, excel中的序号
    pub replace_all: bool,                  // 是否替换所有
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json() {
        let json_data = r#"
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

        let mut expected_lang_map: Vec<(String, String)> = [
            ("zh".to_string(), "中文简体".to_string()),
            ("zh-rTW".to_string(), "中文繁体".to_string()),
            ("en".to_string(), "英语".to_string()),
            ("ja".to_string(), "日语".to_string()),
            ("ko-rKR".to_string(), "韩语".to_string()),
            ("fr".to_string(), "法语".to_string()),
            ("de".to_string(), "德语".to_string()),
            ("es".to_string(), "西班牙语".to_string()),
            ("it".to_string(), "意大利语".to_string()),
            ("nl".to_string(), "荷兰语".to_string()),
        ]
        .to_vec();
        expected_lang_map.sort();
        let expected_config = InputCfg {
            sheet_name: "sheetName".to_string(),
            tag_name: "Android tag".to_string(),
            default_lang: "en".to_string(),
            lang_map: expected_lang_map,
            replace_all: false,
        };
        println!("excepted-->{:?}", expected_config);
        let parsed_config = InputCfg::from_json(json_data).expect("Failed to parse JSON");
        println!("parsed--->{:?}", parsed_config);
        assert_eq!(parsed_config, expected_config);
    }
}
