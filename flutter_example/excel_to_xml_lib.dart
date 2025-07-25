import 'dart:ffi';
import 'dart:io';
import 'dart:convert';

// Define the C function signatures using Pointer<Int8> for C strings
typedef ExcelToXmlUpdateC = Int32 Function(
    Pointer<Int8> cfgJson, Pointer<Int8> excelPath, Pointer<Int8> xmlDirPath);
typedef ExcelToXmlUpdate = int Function(
    Pointer<Int8> cfgJson, Pointer<Int8> excelPath, Pointer<Int8> xmlDirPath);

typedef ExcelToXmlQuickUpdateC = Int32 Function(
    Pointer<Int8> cfgJson, Pointer<Int8> excelPath, Pointer<Int8> xmlDirPath);
typedef ExcelToXmlQuickUpdate = int Function(
    Pointer<Int8> cfgJson, Pointer<Int8> excelPath, Pointer<Int8> xmlDirPath);

typedef ExcelToXmlGetDefaultConfigC = Pointer<Int8> Function();
typedef ExcelToXmlGetDefaultConfig = Pointer<Int8> Function();

typedef ExcelToXmlFreeStringC = Void Function(Pointer<Int8> ptr);
typedef ExcelToXmlFreeString = void Function(Pointer<Int8> ptr);

// Helper functions for string conversion
Pointer<Int8> stringToCString(String str) {
  final units = utf8.encode(str);
  final ptr = allocate<Int8>(count: units.length + 1);
  final nativeString = ptr.asTypedList(units.length + 1);
  nativeString.setAll(0, units);
  nativeString[units.length] = 0;
  return ptr;
}

String cStringToString(Pointer<Int8> ptr) {
  final codeUnits = <int>[];
  var i = 0;
  while (ptr.elementAt(i).value != 0) {
    codeUnits.add(ptr.elementAt(i).value);
    i++;
  }
  return utf8.decode(codeUnits);
}

// Simple memory allocation function
Pointer<T> allocate<T extends NativeType>({required int count}) {
  final size = sizeOf<T>();
  final ptr = calloc<T>(count);
  return ptr;
}

final calloc = DynamicLibrary.process().lookupFunction<
    Pointer Function(IntPtr),
    Pointer Function(int)>('malloc');

class ExcelToXmlLib {
  late DynamicLibrary _lib;
  late ExcelToXmlUpdate _update;
  late ExcelToXmlQuickUpdate _quickUpdate;
  late ExcelToXmlGetDefaultConfig _getDefaultConfig;
  late ExcelToXmlFreeString _freeString;

  ExcelToXmlLib() {
    // Load the dynamic library
    if (Platform.isAndroid) {
      _lib = DynamicLibrary.open('libexcel_to_xml.so');
    } else if (Platform.isIOS) {
      _lib = DynamicLibrary.executable();
    } else if (Platform.isMacOS) {
      _lib = DynamicLibrary.open('libexcel_to_xml.dylib');
    } else if (Platform.isWindows) {
      _lib = DynamicLibrary.open('excel_to_xml.dll');
    } else if (Platform.isLinux) {
      _lib = DynamicLibrary.open('libexcel_to_xml.so');
    } else {
      throw UnsupportedError('Platform ${Platform.operatingSystem} is not supported');
    }

    // Get function pointers
    _update = _lib.lookupFunction<ExcelToXmlUpdateC, ExcelToXmlUpdate>('excel_to_xml_update');
    _quickUpdate = _lib.lookupFunction<ExcelToXmlQuickUpdateC, ExcelToXmlQuickUpdate>('excel_to_xml_quick_update');
    _getDefaultConfig = _lib.lookupFunction<ExcelToXmlGetDefaultConfigC, ExcelToXmlGetDefaultConfig>('excel_to_xml_get_default_config');
    _freeString = _lib.lookupFunction<ExcelToXmlFreeStringC, ExcelToXmlFreeString>('excel_to_xml_free_string');
  }

  /// Update XML files from Excel data
  /// 
  /// Returns true on success, false on failure
  bool update(String cfgJson, String excelPath, String xmlDirPath) {
    final cfgJsonPtr = cfgJson.toNativeUtf8();
    final excelPathPtr = excelPath.toNativeUtf8();
    final xmlDirPathPtr = xmlDirPath.toNativeUtf8();

    try {
      final result = _update(cfgJsonPtr, excelPathPtr, xmlDirPathPtr);
      return result == 0;
    } finally {
      malloc.free(cfgJsonPtr);
      malloc.free(excelPathPtr);
      malloc.free(xmlDirPathPtr);
    }
  }

  /// Quick update XML files from Excel data (uses more memory for better performance)
  /// 
  /// Returns true on success, false on failure
  bool quickUpdate(String cfgJson, String excelPath, String xmlDirPath) {
    final cfgJsonPtr = cfgJson.toNativeUtf8();
    final excelPathPtr = excelPath.toNativeUtf8();
    final xmlDirPathPtr = xmlDirPath.toNativeUtf8();

    try {
      final result = _quickUpdate(cfgJsonPtr, excelPathPtr, xmlDirPathPtr);
      return result == 0;
    } finally {
      malloc.free(cfgJsonPtr);
      malloc.free(excelPathPtr);
      malloc.free(xmlDirPathPtr);
    }
  }

  /// Get the default configuration JSON
  String getDefaultConfig() {
    final configPtr = _getDefaultConfig();
    if (configPtr == nullptr) {
      throw Exception('Failed to get default configuration');
    }

    try {
      return configPtr.toDartString();
    } finally {
      _freeString(configPtr);
    }
  }
}

// Example usage
void main() {
  final lib = ExcelToXmlLib();
  
  // Get default configuration
  final defaultConfig = lib.getDefaultConfig();
  print('Default configuration: $defaultConfig');
  
  // Example update (replace with actual paths)
  // final success = lib.update(
  //   defaultConfig,
  //   '/path/to/excel/file.xlsx',
  //   '/path/to/xml/directory'
  // );
  // print('Update successful: $success');
}
