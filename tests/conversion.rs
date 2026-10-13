use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

static TEST_DIR_ID: AtomicU64 = AtomicU64::new(0);

const ORIGINAL_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="hello">Old</string>
    <string name="fallback">Old fallback</string>
    <string name="empty">Old empty</string>
</resources>
"#;

const MALFORMED_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="hello">未闭合</resources>
"#;

struct TestDir(PathBuf);

impl TestDir {
    fn new(name: &str) -> Self {
        let id = TEST_DIR_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("excel-to-xml-{name}-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).expect("create test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn config_json() -> &'static str {
    r#"{
        "sheetName": "Translations",
        "tagName": "Android tag",
        "defaultLang": "en",
        "langMap": {
            "en": "English",
            "zh": "简体中文"
        },
        "disableEscape": false,
        "escapeOnly": {
            "&": "&amp;",
            ">": "&gt;",
            "<": "&lt;"
        },
        "reset": false,
        "replaceBlankWithDefault": true,
        "regex": "",
        "ignoreFolder": []
    }"#
}

fn write_workbook(path: &Path) {
    write_workbook_with_values(path, "Hello", "Goodbye");
}

fn write_workbook_with_values(path: &Path, hello: &str, bye: &str) {
    let file = File::create(path).expect("create workbook");
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let entries = [
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#,
        ),
        (
            "xl/workbook.xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets><sheet name="Translations" sheetId="1" r:id="rId1"/></sheets>
</workbook>"#,
        ),
        (
            "xl/_rels/workbook.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#,
        ),
        (
            "xl/worksheets/sheet1.xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1">
      <c r="A1" t="inlineStr"><is><t>Android tag</t></is></c>
      <c r="B1" t="inlineStr"><is><t>English</t></is></c>
      <c r="C1" t="inlineStr"><is><t>简体中文</t></is></c>
    </row>
    <row r="2">
      <c r="A2" t="inlineStr"><is><t>hello</t></is></c>
      <c r="B2" t="inlineStr"><is><t>Hello</t></is></c>
      <c r="C2" t="inlineStr"><is><t>你好</t></is></c>
    </row>
    <row r="3">
      <c r="A3" t="inlineStr"><is><t>bye</t></is></c>
      <c r="B3" t="inlineStr"><is><t>Goodbye</t></is></c>
      <c r="C3" t="inlineStr"><is><t>再见</t></is></c>
    </row>
    <row r="4">
      <c r="A4" t="inlineStr"><is><t>fallback</t></is></c>
      <c r="B4" t="inlineStr"><is><t>Fallback</t></is></c>
    </row>
    <row r="5">
      <c r="A5" t="inlineStr"><is><t>empty</t></is></c>
    </row>
    <row r="6">
      <c r="A6" t="inlineStr"><is><t>z_new</t></is></c>
      <c r="B6" t="inlineStr"><is><t>Last</t></is></c>
      <c r="C6" t="inlineStr"><is><t>末项</t></is></c>
    </row>
    <row r="7">
      <c r="A7" t="inlineStr"><is><t>a_new</t></is></c>
      <c r="B7" t="inlineStr"><is><t>First</t></is></c>
      <c r="C7" t="inlineStr"><is><t>首项</t></is></c>
    </row>
  </sheetData>
</worksheet>"#,
        ),
    ];

    for (name, content) in entries {
        zip.start_file(name, options).expect("start workbook entry");
        let content = if name == "xl/worksheets/sheet1.xml" {
            content
                .replace(
                    "<t>Hello</t>",
                    &format!("<t>{}</t>", quick_xml::escape::escape(hello)),
                )
                .replace(
                    "<t>Goodbye</t>",
                    &format!("<t>{}</t>", quick_xml::escape::escape(bye)),
                )
        } else {
            content.to_string()
        };
        zip.write_all(content.as_bytes())
            .expect("write workbook entry");
    }
    zip.finish().expect("finish workbook");
}

fn prepare_module(root: &Path, include_zh: bool, zh_xml: &str) -> PathBuf {
    let module = root.join("module");
    let values = module.join("res/values");
    fs::create_dir_all(&values).expect("create default values directory");
    fs::write(values.join("strings.xml"), ORIGINAL_XML).expect("write default strings");
    if include_zh {
        let values_zh = module.join("res/values-zh");
        fs::create_dir_all(&values_zh).expect("create zh values directory");
        fs::write(values_zh.join("strings.xml"), zh_xml).expect("write zh strings");
    }
    module
}

fn read_strings(module: &Path, language_dir: &str) -> String {
    fs::read_to_string(module.join("res").join(language_dir).join("strings.xml"))
        .expect("read strings.xml")
}

fn assert_no_sidecar_files(path: &Path) {
    for entry in fs::read_dir(path).expect("read directory") {
        let entry = entry.expect("read entry");
        let entry_path = entry.path();
        if entry_path.is_dir() {
            assert_no_sidecar_files(&entry_path);
        } else {
            let name = entry.file_name();
            assert!(
                !name.to_string_lossy().contains(".e2xf-"),
                "unexpected sidecar file: {}",
                entry_path.display()
            );
        }
    }
}

#[test]
fn normal_and_quick_update_produce_the_same_files() {
    let fixture = TestDir::new("matching-modes");
    let workbook = fixture.path().join("translations.xlsx");
    write_workbook(&workbook);
    let normal_module = prepare_module(&fixture.path().join("normal"), true, ORIGINAL_XML);
    let quick_module = prepare_module(&fixture.path().join("quick"), true, ORIGINAL_XML);

    let normal_report = excel_to_xml::convert(
        config_json(),
        workbook.to_str().expect("workbook path"),
        normal_module.to_str().expect("normal module path"),
        excel_to_xml::ConversionMode::LowMemory,
    )
    .expect("normal update succeeds");
    let quick_report = excel_to_xml::convert(
        config_json(),
        workbook.to_str().expect("workbook path"),
        quick_module.to_str().expect("quick module path"),
        excel_to_xml::ConversionMode::Fast,
    )
    .expect("quick update succeeds");
    assert_eq!(normal_report.files_updated, 2);
    assert_eq!(quick_report.files_updated, 2);

    let normal_default = read_strings(&normal_module, "values");
    let quick_default = read_strings(&quick_module, "values");
    let normal_zh = read_strings(&normal_module, "values-zh");
    let quick_zh = read_strings(&quick_module, "values-zh");
    assert_eq!(normal_default, quick_default);
    assert_eq!(normal_zh, quick_zh);
    assert!(normal_default.contains(">Hello</string>"));
    assert!(normal_default.contains("name=\"bye\">Goodbye</string>"));
    assert!(normal_zh.contains(">你好</string>"));
    assert!(normal_zh.contains("name=\"bye\">再见</string>"));
    assert!(normal_zh.contains("name=\"fallback\">Fallback</string>"));
    assert!(normal_zh.contains("name=\"empty\""));
    let first_new_tag = normal_zh.find("name=\"a_new\"").expect("a_new is written");
    let last_new_tag = normal_zh.find("name=\"z_new\"").expect("z_new is written");
    assert!(first_new_tag < last_new_tag);
    assert_no_sidecar_files(fixture.path());
}

#[test]
fn xml_string_bodies_are_replaced_once_in_both_modes() {
    let fixture = TestDir::new("complete-string-replacement");
    let workbook = fixture.path().join("translations.xlsx");
    write_workbook_with_values(&workbook, "Firmware & Battery", "Touch & Go");
    let untouched = r#"<string name="untouched" formatted="false">Keep &amp; &lt; &#x26; <![CDATA[raw & text]]><b>bold</b></string>"#;
    let cases = [
        ("entities", "Firmware &amp; Battery", "Touch &amp; Go"),
        ("entity-only", "&amp;", "&#38;"),
        ("leading-entity", "&amp; Battery", "&#x26; Go"),
        (
            "multiple-entities",
            "A &lt; B &gt; C &amp; D",
            "&#65;&#x42; &quot;C&quot; &apos;D&apos;",
        ),
        (
            "cdata",
            "<![CDATA[Firmware & Battery]]>",
            "<![CDATA[Touch & Go]]>",
        ),
        (
            "mixed-content",
            "Old &amp; <![CDATA[tail]]><b>bold &gt; text</b>",
            "<!--inside-->prefix<i>nested</i><?test instruction?>tail",
        ),
        ("empty", "", ""),
        (
            "already-duplicated",
            "Firmware &amp; Battery&amp; Battery",
            "Touch &amp; Go&amp; Go",
        ),
    ];

    for (case, hello, bye) in cases {
        for (mode_name, mode) in [
            ("normal", excel_to_xml::ConversionMode::LowMemory),
            ("fast", excel_to_xml::ConversionMode::Fast),
        ] {
            let root = fixture.path().join(case).join(mode_name);
            let module = prepare_module(&root, true, ORIGINAL_XML);
            let xml = format!(
                r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <!--outside-->
    <string name="hello" translatable="false">{hello}</string>
    <string name="bye" formatted="false">{bye}</string>
    {untouched}
    <string name="untouched_empty"/>
</resources>
"#
            );
            fs::write(module.join("res/values/strings.xml"), xml).expect("write source XML");
            let mut first_result = None;
            for pass in 1..=2 {
                let report = excel_to_xml::convert(
                    config_json(),
                    workbook.to_str().expect("workbook path"),
                    module.to_str().expect("module path"),
                    mode,
                )
                .expect("conversion succeeds");
                assert_eq!(report.files_updated, 2);
                let output = read_strings(&module, "values");
                assert!(
                    output.contains(r#"<string name="hello" translatable="false">Firmware &amp; Battery</string>"#),
                    "{case}/{mode_name}/pass-{pass}: {output}"
                );
                assert!(
                    output.contains(
                        r#"<string name="bye" formatted="false">Touch &amp; Go</string>"#
                    ),
                    "{case}/{mode_name}/pass-{pass}: {output}"
                );
                assert_eq!(output.matches("name=\"hello\"").count(), 1);
                assert_eq!(output.matches("name=\"bye\"").count(), 1);
                assert!(output.contains(untouched));
                assert!(output.contains(r#"<string name="untouched_empty"/>"#));
                assert!(output.contains("<!--outside-->"));
                if let Some(first) = &first_result {
                    assert_eq!(&output, first, "{case}/{mode_name} must be idempotent");
                } else {
                    first_result = Some(output);
                }
            }
        }
    }
    assert_no_sidecar_files(fixture.path());
}

#[test]
fn self_closing_strings_are_updated_without_duplicate_names() {
    let fixture = TestDir::new("self-closing-strings");
    let workbook = fixture.path().join("translations.xlsx");
    write_workbook_with_values(&workbook, "Firmware & Battery", "Touch & Go");
    for (mode_name, mode) in [
        ("normal", excel_to_xml::ConversionMode::LowMemory),
        ("fast", excel_to_xml::ConversionMode::Fast),
    ] {
        let module = prepare_module(&fixture.path().join(mode_name), true, ORIGINAL_XML);
        fs::write(
            module.join("res/values/strings.xml"),
            r#"<resources>
    <string name="hello" translatable="false"/>
    <string name="bye" formatted="false"/>
    <string name="empty" formatted="false"/>
    <string name="untouched_empty"/>
</resources>"#,
        )
        .expect("write self-closing strings");
        let mut first_result = None;
        for _ in 0..2 {
            excel_to_xml::convert(
                config_json(),
                workbook.to_str().expect("workbook path"),
                module.to_str().expect("module path"),
                mode,
            )
            .expect("conversion succeeds");
            let output = read_strings(&module, "values");
            assert!(output.contains(
                r#"<string name="hello" translatable="false">Firmware &amp; Battery</string>"#
            ));
            assert!(
                output.contains(r#"<string name="bye" formatted="false">Touch &amp; Go</string>"#)
            );
            assert!(output.contains(r#"<string name="empty" formatted="false"></string>"#));
            for tag in ["hello", "bye", "empty", "untouched_empty"] {
                assert_eq!(output.matches(&format!("name=\"{tag}\"")).count(), 1);
            }
            assert!(output.contains(r#"<string name="untouched_empty"/>"#));
            if let Some(first) = &first_result {
                assert_eq!(&output, first);
            } else {
                first_result = Some(output);
            }
        }
    }
    assert_no_sidecar_files(fixture.path());
}

#[test]
fn missing_language_file_is_an_error_without_modifying_existing_files() {
    let fixture = TestDir::new("missing-language");
    let workbook = fixture.path().join("translations.xlsx");
    write_workbook(&workbook);
    let module = prepare_module(fixture.path(), false, ORIGINAL_XML);

    let error = excel_to_xml::convert(
        config_json(),
        workbook.to_str().expect("workbook path"),
        module.to_str().expect("module path"),
        excel_to_xml::ConversionMode::Fast,
    )
    .expect_err("missing zh file must fail");

    assert!(matches!(
        &error,
        excel_to_xml::ConversionError::TargetResolution(_)
    ));
    assert!(
        error.to_string().contains("缺少语言 zh 的目标文件"),
        "{error}"
    );
    assert_eq!(read_strings(&module, "values"), ORIGINAL_XML);
    assert_no_sidecar_files(fixture.path());
}

#[test]
fn malformed_second_xml_does_not_partially_update_the_first_file() {
    let fixture = TestDir::new("staged-failure");
    let workbook = fixture.path().join("translations.xlsx");
    write_workbook(&workbook);
    let module = prepare_module(fixture.path(), true, MALFORMED_XML);

    let error = excel_to_xml::convert(
        config_json(),
        workbook.to_str().expect("workbook path"),
        module.to_str().expect("module path"),
        excel_to_xml::ConversionMode::Fast,
    )
    .expect_err("malformed XML must fail");

    assert!(matches!(&error, excel_to_xml::ConversionError::Xml(_)));
    assert!(error.to_string().contains("XML 错误"), "{error}");
    assert_eq!(read_strings(&module, "values"), ORIGINAL_XML);
    assert_eq!(read_strings(&module, "values-zh"), MALFORMED_XML);
    assert_no_sidecar_files(fixture.path());
}

#[test]
fn sheet_name_errors_are_preserved() {
    let fixture = TestDir::new("sheet-error");
    let missing = fixture.path().join("missing.xlsx");

    let error = excel_to_xml::get_sheet_names(missing.to_str().expect("missing path"))
        .expect_err("missing workbook must fail");

    assert!(matches!(&error, excel_to_xml::ConversionError::Excel(_)));
    assert!(
        error.to_string().contains("读取 Excel 工作表失败"),
        "{error}"
    );
}
