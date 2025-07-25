# Flutter集成快速开始指南

## 1. 准备工作

### 1.1 构建Rust库
```bash
# 在项目根目录执行
./build_for_flutter.sh
```

这将在`flutter_lib/`目录下生成：
- `libexcel_to_xml.dylib` (macOS)
- `libexcel_to_xml.so` (Linux)  
- `excel_to_xml.dll` (Windows)

### 1.2 Flutter项目配置

在您的Flutter项目的`pubspec.yaml`中添加：
```yaml
dependencies:
  ffi: ^2.0.0
```

## 2. 库文件部署

### 2.1 桌面平台 (macOS/Windows/Linux)
将库文件复制到Flutter项目根目录的`lib/native/`文件夹中：
```
your_flutter_project/
  lib/
    native/
      libexcel_to_xml.dylib    # macOS
      libexcel_to_xml.so       # Linux  
      excel_to_xml.dll         # Windows
```

### 2.2 移动平台
- **Android**: 将`.so`文件放在`android/app/src/main/jniLibs/arm64-v8a/`
- **iOS**: 需要先构建静态库并添加到iOS项目中

## 3. 使用示例

### 3.1 基本用法

```dart
import 'dart:ffi';
import 'dart:io';

class ExcelToXmlLib {
  static DynamicLibrary? _lib;
  
  static DynamicLibrary get lib {
    if (_lib != null) return _lib!;
    
    if (Platform.isMacOS) {
      _lib = DynamicLibrary.open('lib/native/libexcel_to_xml.dylib');
    } else if (Platform.isWindows) {
      _lib = DynamicLibrary.open('lib/native/excel_to_xml.dll');
    } else if (Platform.isLinux) {
      _lib = DynamicLibrary.open('lib/native/libexcel_to_xml.so');
    } else if (Platform.isAndroid) {
      _lib = DynamicLibrary.open('libexcel_to_xml.so');
    } else if (Platform.isIOS) {
      _lib = DynamicLibrary.executable();
    } else {
      throw UnsupportedError('Platform not supported');
    }
    
    return _lib!;
  }
  
  // 获取默认配置
  static String getDefaultConfig() {
    final getConfigFunc = lib.lookupFunction<
        Pointer<Int8> Function(),
        Pointer<Int8> Function()>('excel_to_xml_get_default_config');
    
    final freeStringFunc = lib.lookupFunction<
        Void Function(Pointer<Int8>),
        void Function(Pointer<Int8>)>('excel_to_xml_free_string');
    
    final configPtr = getConfigFunc();
    if (configPtr == nullptr) {
      throw Exception('Failed to get default configuration');
    }
    
    try {
      // 手动转换C字符串为Dart字符串
      final bytes = <int>[];
      var i = 0;
      while (configPtr.elementAt(i).value != 0) {
        bytes.add(configPtr.elementAt(i).value);
        i++;
      }
      return String.fromCharCodes(bytes);
    } finally {
      freeStringFunc(configPtr);
    }
  }
  
  // 更新XML文件
  static bool update(String configJson, String excelPath, String xmlDirPath) {
    // 这里需要实现C字符串转换
    // 暂时返回false作为占位符
    print('update called with: $configJson, $excelPath, $xmlDirPath');
    return false;
  }
}

// 使用示例
void main() {
  try {
    final config = ExcelToXmlLib.getDefaultConfig();
    print('默认配置: $config');
    
    final success = ExcelToXmlLib.update(
      config,
      '/path/to/your/excel.xlsx',
      '/path/to/your/android/project'
    );
    
    print('更新结果: ${success ? "成功" : "失败"}');
  } catch (e) {
    print('错误: $e');
  }
}
```

## 4. 配置说明

默认配置示例：
```json
{
    "sheetName": "",
    "tagName": "Android tag", 
    "defaultLang": "en",
    "langMap": {
        "zh": "中文简体",
        "zh-rTW": "中文繁体", 
        "en": "英语",
        "ja": "日语",
        "ko-rKR": "韩语"
    },
    "disableEscape": false,
    "reset": false,
    "replaceBlankWithDefault": true,
    "regex": "",
    "ignoreFolder": ["build"],
    "targetFolder": "res"
}
```

### 配置字段说明：
- `sheetName`: Excel工作表名称，空字符串表示使用第一个工作表
- `tagName`: 包含Android字符串标签的列名
- `defaultLang`: 默认语言代码
- `langMap`: 语言代码到语言名称的映射
- `disableEscape`: 是否禁用XML字符转义
- `reset`: 是否重置整个XML文件（而不是更新现有标签）
- `replaceBlankWithDefault`: 空值是否用默认语言替换
- `regex`: 用于过滤内容的正则表达式
- `ignoreFolder`: 搜索时忽略的文件夹
- `targetFolder`: 包含字符串资源的目标文件夹名

## 5. 错误处理

函数返回值说明：
- `0`: 成功
- `-1`: cfg_json参数编码错误  
- `-2`: excel_path参数编码错误
- `-3`: xml_dir_path参数编码错误
- `-4`: 更新操作失败

## 6. 注意事项

1. 确保Excel文件格式正确，包含必要的列
2. 确保有XML目录的写入权限
3. 库是线程安全的，但建议在主线程调用
4. 记得释放库分配的字符串内存（使用`excel_to_xml_free_string`）
5. 第一次使用前建议先测试基本功能

## 7. 故障排除

### 常见问题：

**问题**: 无法加载动态库
**解决**: 
- 检查库文件路径是否正确
- 确保库文件有执行权限
- 在macOS上，可能需要解除Gatekeeper限制

**问题**: 函数调用失败
**解决**:
- 检查参数是否正确编码为UTF-8
- 确保路径存在且有权限
- 查看控制台输出的错误信息

**问题**: Excel文件无法解析  
**解决**:
- 确保Excel文件格式正确
- 检查配置中的列名是否匹配
- 确保文件没有被其他程序占用
