use std::ffi::{CStr, CString};

// 测试FFI函数
fn main() {
    println!("Testing FFI functions...");
    
    // Test getting default config
    let config_ptr = unsafe { excel_to_xml_get_default_config() };
    if !config_ptr.is_null() {
        let config_str = unsafe { CStr::from_ptr(config_ptr) };
        match config_str.to_str() {
            Ok(config) => {
                println!("Default config retrieved successfully:");
                println!("{}", config);
            },
            Err(e) => {
                println!("Failed to convert config to string: {:?}", e);
            }
        }
        
        // Free the string
        unsafe { excel_to_xml_free_string(config_ptr) };
    } else {
        println!("Failed to get default config");
    }
    
    // Test update function with dummy data
    let cfg_json = CString::new(r#"{"test": "config"}"#).unwrap();
    let excel_path = CString::new("/dummy/path.xlsx").unwrap();
    let xml_dir_path = CString::new("/dummy/xml/dir").unwrap();
    
    let result = unsafe {
        excel_to_xml_update(
            cfg_json.as_ptr(),
            excel_path.as_ptr(),
            xml_dir_path.as_ptr()
        )
    };
    
    println!("Update function returned: {}", result);
    println!("Test completed!");
}

// Import the FFI functions
extern "C" {
    fn excel_to_xml_get_default_config() -> *mut std::os::raw::c_char;
    fn excel_to_xml_free_string(ptr: *mut std::os::raw::c_char);
    fn excel_to_xml_update(
        cfg_json: *const std::os::raw::c_char,
        excel_path: *const std::os::raw::c_char,
        xml_dir_path: *const std::os::raw::c_char,
    ) -> i32;
}
