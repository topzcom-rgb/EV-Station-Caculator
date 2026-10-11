/// ระบบดาวน์โหลด/บันทึกไฟล์แบบ Abstract รองรับทั้ง Desktop (Native File Dialog / Disk Save)
/// และ Web (WebAssembly Browser Blob Download)

#[cfg(not(target_arch = "wasm32"))]
pub fn trigger_download(filename: &str, content: &[u8], _mime_type: &str) -> Result<Option<std::path::PathBuf>, String> {
    let extension = std::path::Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let filter_name = match extension {
        "xlsx" => "Excel Workbook (*.xlsx)",
        "dxf" => "AutoCAD DXF (*.dxf)",
        _ => "All Files (*.*)",
    };

    let mut dialog = rfd::FileDialog::new().set_file_name(filename);
    if !extension.is_empty() {
        dialog = dialog.add_filter(filter_name, &[extension]);
    }

    if let Some(path) = dialog.save_file() {
        std::fs::write(&path, content)
            .map_err(|e| format!("Failed to write file to {}: {}", path.display(), e))?;
        Ok(Some(path))
    } else {
        // ผู้ใช้กดยกเลิก Save Dialog
        Ok(None)
    }
}

#[cfg(target_arch = "wasm32")]
pub fn trigger_download(filename: &str, content: &[u8], mime_type: &str) -> Result<Option<std::path::PathBuf>, String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or_else(|| "No global `window` exists".to_string())?;
    let document = window.document().ok_or_else(|| "No `document` on window".to_string())?;

    // 1. สร้าง uint8 array จาก slice ของ content
    let uint8_array = js_sys::Uint8Array::from(content);
    let array = js_sys::Array::new();
    array.push(&uint8_array);

    // 2. กำหนด BlobPropertyBag และสร้าง Blob
    let blob_props = web_sys::BlobPropertyBag::new();
    blob_props.set_type(mime_type);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&array, &blob_props)
        .map_err(|e| format!("Failed to create Blob: {:?}", e))?;

    // 3. สร้าง Object URL
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("Failed to create object URL: {:?}", e))?;

    // 4. สร้างแท็ก <a> เพื่อสั่ง download
    let link: web_sys::HtmlElement = document
        .create_element("a")
        .map_err(|e| format!("Failed to create <a> element: {:?}", e))?
        .dyn_into()
        .map_err(|_| "Failed to cast element to HtmlElement".to_string())?;

    link.set_attribute("href", &url)
        .map_err(|e| format!("Failed to set href attribute: {:?}", e))?;
    link.set_attribute("download", filename)
        .map_err(|e| format!("Failed to set download attribute: {:?}", e))?;

    let body = document.body().ok_or_else(|| "No `body` on document".to_string())?;
    body.append_child(&link)
        .map_err(|e| format!("Failed to append link: {:?}", e))?;

    link.click();

    body.remove_child(&link)
        .map_err(|e| format!("Failed to remove link: {:?}", e))?;

    // ป้องกันการ Revoke Object URL ทันทีใน Tick เดียวกัน ซึ่งอาจทำให้ Browser (Chrome/Safari) ยกเลิกการดาวน์โหลด
    let url_clone = url.clone();
    let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
        let _ = web_sys::Url::revoke_object_url(&url_clone);
    });
    if let Ok(func) = cb.dyn_into::<js_sys::Function>() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&func, 2000);
    }

    Ok(Some(std::path::PathBuf::from(filename)))
}
