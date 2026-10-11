use std::path::PathBuf;

/// เปิด Native Save Dialog สำหรับเลือกตำแหน่งและตั้งชื่อไฟล์เพื่อบันทึก
///
/// พารามิเตอร์:
/// - `default_name`: ชื่อไฟล์เริ่มต้น เช่น `"Load_Schedule.xlsx"` หรือ `"SLD_Diagram.dxf"`
/// - `extension`: นามสกุลไฟล์ เช่น `"xlsx"` หรือ `"dxf"`
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_save_path(default_name: &str, extension: &str) -> Option<PathBuf> {
    let filter_name = match extension {
        "xlsx" => "Excel Workbook (*.xlsx)",
        "dxf" => "AutoCAD DXF (*.dxf)",
        _ => "All Files (*.*)",
    };

    rfd::FileDialog::new()
        .set_file_name(default_name)
        .add_filter(filter_name, &[extension])
        .save_file()
}

#[cfg(target_arch = "wasm32")]
pub fn pick_save_path(_default_name: &str, _extension: &str) -> Option<PathBuf> {
    None
}
