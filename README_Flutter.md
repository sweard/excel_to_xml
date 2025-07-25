# Excel to XML Library for Flutter

这个项目将Rust库转换为可供Flutter应用调用的动态库。

## 项目结构

```
excel_to_xml/
├── src/
│   ├── lib.rs          # FFI接口层，供Flutter调用
│   ├── main.rs         # 原始的命令行应用
│   ├── write_xml.rs    # XML写入功能
│   ├── read_excel.rs   # Excel读取功能
│   ├── find_files.rs   # 文件查找功能
│   └── config.rs       # 配置管理
├── include/
│   └── excel_to_xml.h  # C头文件，定义FFI接口
├── flutter_example/
│   ├── excel_to_xml_lib_simple.dart  # Flutter集成示例
│   └── excel_to_xml_lib.dart         # 完整的Flutter集成代码（需要完善）
├── flutter_lib/        # 构建输出目录
├── build_for_flutter.sh # 构建脚本
├── Cargo.toml          # Rust项目配置
└── README.md           # 本文件
```

## 功能特性

- 从Excel文件读取多语言数据
- 更新Android XML字符串资源文件
- 支持多种语言和配置选项
- 提供C FFI接口供Flutter调用
- 支持普通更新和快速更新（内存换时间）

## 构建步骤

### 1. 构建Rust库

```bash
# 给构建脚本添加执行权限
chmod +x build_for_flutter.sh

# 执行构建脚本
./build_for_flutter.sh
```

这将为当前平台构建动态库：
- macOS: `libexcel_to_xml.dylib`
- Linux: `libexcel_to_xml.so`
- Windows: `excel_to_xml.dll`

### 2. Flutter集成

1. 将生成的库文件复制到Flutter项目中：
   - Android: `android/app/src/main/jniLibs/arm64-v8a/` (或相应架构)
   - iOS: 将`.a`文件添加到iOS项目
   - macOS/Windows/Linux: 将库文件放在可访问的路径

2. 在Flutter项目的`pubspec.yaml`中添加FFI依赖：
   ```yaml
   dependencies:
     ffi: ^2.0.0
   ```

3. 使用示例代码 `flutter_example/excel_to_xml_lib_simple.dart`

## FFI接口

库提供以下C接口：

### `excel_to_xml_update`
```c
int excel_to_xml_update(const char* cfg_json, const char* excel_path, const char* xml_dir_path);
```
更新XML文件。返回0表示成功，负数表示错误。

### `excel_to_xml_quick_update`
```c
int excel_to_xml_quick_update(const char* cfg_json, const char* excel_path, const char* xml_dir_path);
```
快速更新XML文件（使用更多内存）。返回0表示成功，负数表示错误。

### `excel_to_xml_get_default_config`
```c
char* excel_to_xml_get_default_config(void);
```
获取默认配置JSON字符串。返回的字符串必须使用`excel_to_xml_free_string`释放。

### `excel_to_xml_free_string`
```c
void excel_to_xml_free_string(char* ptr);
```
释放由库分配的字符串内存。

## 配置格式

配置使用JSON格式：

```json
{
    "sheetName": "",                    // Excel工作表名称（空字符串使用第一个）
    "tagName": "Android tag",           // 标签名称列的标题
    "defaultLang": "en",                // 默认语言
    "langMap": {                        // 语言映射
        "zh": "中文简体",
        "en": "英语"
    },
    "disableEscape": false,             // 是否禁用XML转义
    "escapeOnly": {},                   // 仅转义指定字符
    "reset": false,                     // 是否重置整个XML文件
    "replaceBlankWithDefault": true,    // 空值是否用默认语言替换
    "regex": "",                        // 正则表达式过滤
    "ignoreFolder": ["build"],          // 忽略的文件夹
    "targetFolder": "res"               // 目标文件夹名称
}
```

## 使用示例

### Dart/Flutter代码

```dart
import 'excel_to_xml_lib_simple.dart';

void main() {
  final lib = ExcelToXmlLib();
  
  // 获取默认配置
  final config = lib.getDefaultConfig();
  
  // 更新XML文件
  final success = lib.update(
    config,
    '/path/to/your/excel/file.xlsx',
    '/path/to/your/android/project'
  );
  
  if (success) {
    print('XML文件更新成功');
  } else {
    print('XML文件更新失败');
  }
}
```

## 错误代码

- `0`: 成功
- `-1`: cfg_json参数UTF-8编码错误
- `-2`: excel_path参数UTF-8编码错误
- `-3`: xml_dir_path参数UTF-8编码错误
- `-4`: 更新操作失败

## 注意事项

1. 确保Excel文件格式正确，包含所需的列
2. 确保XML目录路径存在且有写入权限
3. 库函数是线程安全的，但建议在主线程调用
4. 记得释放由库分配的字符串内存

## 开发说明

- 使用`cargo build --release`构建优化版本
- 使用`cargo test`运行测试
- FFI接口在`src/lib.rs`中定义
- 原始功能在其他模块中实现

## 许可证

[添加你的许可证信息]
