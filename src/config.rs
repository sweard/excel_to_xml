use serde_json::{from_str, Value};
use std::error::Error;

pub const CFG_JSON: &str = r#"{
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
    "disableEscape": false,
    "escapeOnly":{
        "\\\n":"\\n",
        "\n":"\\n",
        "\\\\'":"\\'",
        "'":"\\'",
        "\\\\\"":"\\\"",
        "\"":"\\\"",
        " ":" ",
        "&":"&amp;",
        ">":"&gt;",
        "<":"&lt;"
    },
    "reset": false,
    "replaceBlankWithDefault": true,
    "regex":"^\\s+|\\s+$"
}"#;

/**
 * 解析完excel后生成的配置
 */
#[derive(Debug, PartialEq)]
pub struct ParsedCfg {
    pub sheet_name: String,                 // 表名
    pub default_lang: String,               // 默认语言
    pub reset: bool,                        // 是否替换所有
    pub disable_escape: bool,               // 是否禁用转义
    pub escape_only: Vec<(String, String)>, // 只需要转义这部分内容，没配置就转义全部
    pub replace_blank_with_default: bool,   // 是否替换空白内容为默认语言
    pub regex: String,                      // 正则表达式

    // 输入配置字段 (原InputCfg特有)
    pub tag_name: String,                // 标签列名称
    pub lang_map: Vec<(String, String)>, // 语言名称 zh - 简体中文

    // 解析后的字段 (原ParsedCfg特有)
    pub tag_index: u32,                     // 标签序号 excel中的序号
    pub lang_index_map: Vec<(String, u32)>, // 语言名称 zh - 0（excel中的序号）
}

impl ParsedCfg {
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
        let reset = json_obj
            .get("reset")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let disable_escape = json_obj
            .get("disableEscape")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let escape_only = json_obj
            .get("escapeOnly")
            .and_then(Value::as_object)
            .ok_or("Missing or invalid 'escapeOnly' field")?
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect();

        let replace_blank_with_default = json_obj
            .get("replaceBlankWithDefault")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let regex = json_obj
            .get("regex")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        Ok(ParsedCfg {
            sheet_name,
            tag_name,
            default_lang,
            lang_map,
            reset,
            disable_escape,
            escape_only,
            replace_blank_with_default,
            regex,
            tag_index: 0,           // 默认值
            lang_index_map: vec![], // 默认值
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json() {
        let json_data = CFG_JSON;

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

        let mut expected_escape_only: Vec<(String, String)> = [
            ("\n".to_string(), "\\n".to_string()),
            ("\\\\n".to_string(), "\\n".to_string()),
            ("'".to_string(), "\\'".to_string()),
            ("\\\\'".to_string(), "\\'".to_string()),
            ("\"".to_string(), "\\\"".to_string()),
            ("\\\\\"".to_string(), "\\\"".to_string()),
            (" ".to_string(), " ".to_string()),
        ]
        .to_vec();
        expected_escape_only.sort();

        let expected_config = ParsedCfg {
            sheet_name: "sheetName".to_string(),
            tag_name: "Android tag".to_string(),
            default_lang: "en".to_string(),
            lang_map: expected_lang_map,
            reset: false,
            disable_escape: false,
            escape_only: expected_escape_only,
            replace_blank_with_default: true,
            regex: "".to_string(),
            tag_index: 0,
            lang_index_map: vec![],
        };
        println!("excepted-->{:?}", expected_config);
        let parsed_config = ParsedCfg::from_json(json_data).expect("Failed to parse JSON");
        println!("parsed--->{:?}", parsed_config);
        assert_eq!(parsed_config, expected_config);
    }
}
