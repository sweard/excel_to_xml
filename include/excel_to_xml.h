#ifndef EXCEL_TO_XML_H
#define EXCEL_TO_XML_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Update XML files from Excel data.
 * 
 * @param cfg_json Configuration JSON string
 * @param excel_path Path to the Excel file
 * @param xml_dir_path Path to the XML directory
 * @return 0 on success, negative value on error:
 *         -1: Invalid UTF-8 in cfg_json
 *         -2: Invalid UTF-8 in excel_path
 *         -3: Invalid UTF-8 in xml_dir_path
 *         -4: Update operation failed
 */
int excel_to_xml_update(const char* cfg_json, const char* excel_path, const char* xml_dir_path);

/**
 * Quick update XML files from Excel data (uses more memory for better performance).
 * 
 * @param cfg_json Configuration JSON string
 * @param excel_path Path to the Excel file
 * @param xml_dir_path Path to the XML directory
 * @return 0 on success, negative value on error:
 *         -1: Invalid UTF-8 in cfg_json
 *         -2: Invalid UTF-8 in excel_path
 *         -3: Invalid UTF-8 in xml_dir_path
 *         -4: Update operation failed
 */
int excel_to_xml_quick_update(const char* cfg_json, const char* excel_path, const char* xml_dir_path);

/**
 * Get the default configuration JSON string.
 * 
 * @return Pointer to a null-terminated string containing the default configuration.
 *         Must be freed using excel_to_xml_free_string().
 *         Returns NULL on error.
 */
char* excel_to_xml_get_default_config(void);

/**
 * Free a string allocated by this library.
 * 
 * @param ptr Pointer to the string to free. Must have been allocated by this library.
 */
void excel_to_xml_free_string(char* ptr);

#ifdef __cplusplus
}
#endif

#endif /* EXCEL_TO_XML_H */
