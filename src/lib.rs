use std::ffi::{CStr, CString};
use std::os::raw::c_char;

mod config;
mod find_files;
mod read_excel;
mod write_xml;

/// FFI wrapper for the update function
/// 
/// # Safety
/// 
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that:
/// - `cfg_json`, `excel_path`, and `xml_dir_path` are valid null-terminated C strings
/// - The pointers remain valid for the duration of the call
/// - The strings are properly encoded UTF-8
#[no_mangle]
pub unsafe extern "C" fn excel_to_xml_update(
    cfg_json: *const c_char,
    excel_path: *const c_char,
    xml_dir_path: *const c_char,
) -> i32 {
    // Convert C strings to Rust strings
    let cfg_json = match CStr::from_ptr(cfg_json).to_str() {
        Ok(s) => s,
        Err(_) => return -1, // Invalid UTF-8 in cfg_json
    };
    
    let excel_path = match CStr::from_ptr(excel_path).to_str() {
        Ok(s) => s,
        Err(_) => return -2, // Invalid UTF-8 in excel_path
    };
    
    let xml_dir_path = match CStr::from_ptr(xml_dir_path).to_str() {
        Ok(s) => s,
        Err(_) => return -3, // Invalid UTF-8 in xml_dir_path
    };

    // Call the actual update function
    match write_xml::update(cfg_json, excel_path, xml_dir_path) {
        Ok(_) => 0,    // Success
        Err(_) => -4,  // Update failed
    }
}

/// FFI wrapper for the quick_update function
/// 
/// # Safety
/// 
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that:
/// - `cfg_json`, `excel_path`, and `xml_dir_path` are valid null-terminated C strings
/// - The pointers remain valid for the duration of the call
/// - The strings are properly encoded UTF-8
#[no_mangle]
pub unsafe extern "C" fn excel_to_xml_quick_update(
    cfg_json: *const c_char,
    excel_path: *const c_char,
    xml_dir_path: *const c_char,
) -> i32 {
    // Convert C strings to Rust strings
    let cfg_json = match CStr::from_ptr(cfg_json).to_str() {
        Ok(s) => s,
        Err(_) => return -1, // Invalid UTF-8 in cfg_json
    };
    
    let excel_path = match CStr::from_ptr(excel_path).to_str() {
        Ok(s) => s,
        Err(_) => return -2, // Invalid UTF-8 in excel_path
    };
    
    let xml_dir_path = match CStr::from_ptr(xml_dir_path).to_str() {
        Ok(s) => s,
        Err(_) => return -3, // Invalid UTF-8 in xml_dir_path
    };

    // Call the actual quick_update function
    match write_xml::quick_update(cfg_json, excel_path, xml_dir_path) {
        Ok(_) => 0,    // Success
        Err(_) => -4,  // Update failed
    }
}

/// Get the default configuration JSON
/// 
/// # Safety
/// 
/// This function returns a pointer to a C string that must be freed by calling `free_string`.
/// The caller is responsible for freeing the memory.
#[no_mangle]
pub extern "C" fn excel_to_xml_get_default_config() -> *mut c_char {
    let config_json = config::CFG_JSON;
    match CString::new(config_json) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string allocated by this library
/// 
/// # Safety
/// 
/// This function is unsafe because it deallocates memory.
/// The caller must ensure that:
/// - `ptr` was allocated by this library (e.g., returned from `excel_to_xml_get_default_config`)
/// - `ptr` is not used after calling this function
/// - `ptr` is not null
#[no_mangle]
pub unsafe extern "C" fn excel_to_xml_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    let _ = CString::from_raw(ptr);
}

// Re-export the main functions for use as a Rust library
pub use write_xml::{update, quick_update};
pub use config::CFG_JSON;

// For testing and direct Rust usage
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = excel_to_xml_get_default_config();
        assert!(!config.is_null());
        unsafe {
            excel_to_xml_free_string(config);
        }
    }
}
