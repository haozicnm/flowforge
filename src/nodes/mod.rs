pub mod traits;
pub mod registry;
pub mod http_node;
pub mod shell_node;
pub mod delay_node;
pub mod script_node;
pub mod webhook_node;
pub mod log_node;

// Flow control
pub mod condition_node;
pub mod loop_node;
pub mod try_catch_node;

// Data operations
pub mod variable_node;
pub mod json_node;
pub mod regex_node;
pub mod template_node;

// Web automation (via WebBridge)
pub mod webbridge;
pub mod web_navigate_node;
pub mod web_click_node;
pub mod web_input_node;
pub mod web_extract_node;
pub mod web_screenshot_node;
pub mod web_wait_node;

// Excel
pub mod excel_read_node;
pub mod excel_write_node;

// Word (.docx)
pub mod docx_read_node;
pub mod docx_create_node;

// Database
pub mod database_node;

// Notification
pub mod notification_node;

// Email
pub mod email_send_node;
pub mod email_read_node;

// FTP
pub mod ftp_upload_node;
pub mod ftp_download_node;

// Media
pub mod image_process_node;

// Document
pub mod pdf_extract_node;

// File operations
pub mod file_node;

// Cron / schedule
pub mod cron_node;
pub mod transform_node;
