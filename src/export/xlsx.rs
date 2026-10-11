use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook};
use std::path::Path;

use crate::StationOutput;

/// ส่งออกตาราง Load Schedule เป็นไฟล์ Excel (.xlsx) ตามมาตรฐานวิศวกรรมไฟฟ้าและเอกสารอ้างอิง
/// สร้างข้อมูลไบต์ของไฟล์ Excel (.xlsx) สำหรับตาราง Load Schedule ในหน่วยความจำ (In-memory)
pub fn export_load_schedule_bytes(
    output: &StationOutput,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Load Schedule")?;

    // --- สไตล์และฟอร์แมต ---
    let font_name = "TH Sarabun New";

    // 1. หัวตารางหลัก (Title Format)
    let title_format = Format::new()
        .set_font_name(font_name)
        .set_font_size(18)
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);

    // 2. ข้อมูลโครงการและหม้อแปลง (Meta Format)
    let meta_bold = Format::new()
        .set_font_name(font_name)
        .set_font_size(13)
        .set_bold();

    let meta_normal = Format::new().set_font_name(font_name).set_font_size(13);

    // 3. Header ตาราง (Table Header Format)
    let th_format = Format::new()
        .set_font_name(font_name)
        .set_font_size(12)
        .set_bold()
        .set_background_color(Color::RGB(0x1B365D)) // Deep Navy
        .set_font_color(Color::White)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    // 4. ข้อมูลทั่วไป (Data Cells)
    let cell_center = Format::new()
        .set_font_name(font_name)
        .set_font_size(12)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    let cell_left = Format::new()
        .set_font_name(font_name)
        .set_font_size(12)
        .set_align(FormatAlign::Left)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    let cell_number = Format::new()
        .set_font_name(font_name)
        .set_font_size(12)
        .set_num_format("#,##0")
        .set_align(FormatAlign::Right)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    let cell_decimal = Format::new()
        .set_font_name(font_name)
        .set_font_size(12)
        .set_num_format("0.00")
        .set_align(FormatAlign::Right)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    // 5. สรุปรวม (Summary / Total Rows)
    let total_label_format = Format::new()
        .set_font_name(font_name)
        .set_font_size(13)
        .set_bold()
        .set_background_color(Color::RGB(0xEAEEF3))
        .set_align(FormatAlign::Right)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    let total_number_format = Format::new()
        .set_font_name(font_name)
        .set_font_size(13)
        .set_bold()
        .set_background_color(Color::RGB(0xEAEEF3))
        .set_num_format("#,##0")
        .set_align(FormatAlign::Right)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    // --- ส่วนที่ 1: Header Section ข้อมูลหัวโครงการ ---
    worksheet.merge_range(
        0,
        0,
        0,
        11,
        "ELECTRICAL LOAD SCHEDULE & SIZING SPECIFICATION",
        &title_format,
    )?;
    worksheet.merge_range(
        1,
        0,
        1,
        11,
        "EV CHARGING STATION (PEA / EIT STANDARD IEC 60364 & IEC 61439-1)",
        &Format::new()
            .set_font_name(font_name)
            .set_font_size(14)
            .set_italic()
            .set_align(FormatAlign::Center),
    )?;

    worksheet.write_string_with_format(3, 0, "หม้อแปลงไฟฟ้า (Transformer):", &meta_bold)?;
    let tx_info = format!(
        "{:.0} kVA, 22 kV / 400-230 V (พิกัดกระแส {:.1} A)",
        output.transformer.kva, output.transformer.rated_current_a
    );
    worksheet.write_string_with_format(3, 2, &tx_info, &meta_normal)?;

    worksheet.write_string_with_format(3, 7, "ระบบไฟฟ้า (System):", &meta_bold)?;
    let sys_info = format!(
        "3 Phase 4 Wire, {:.0} V 50 Hz",
        output.load_summary.system_voltage_v
    );
    worksheet.write_string_with_format(3, 9, &sys_info, &meta_normal)?;

    worksheet.write_string_with_format(4, 0, "ตู้ประธานไฟฟ้า (MDB Main Breaker):", &meta_bold)?;
    worksheet.write_string_with_format(4, 2, &output.main_breaker.name, &meta_normal)?;

    worksheet.write_string_with_format(4, 7, "สายป้อนเมน (Main Feeder):", &meta_bold)?;
    worksheet.write_string_with_format(4, 9, &output.main_cable.cable_spec.description, &meta_normal)?;

    // --- ส่วนที่ 2: หัวตาราง (Table Columns Headers) ---
    let header_row: u32 = 6;
    let headers = [
        "CKT No.",
        "Description (รายการโหลด)",
        "Phase A (VA)",
        "Phase B (VA)",
        "Phase C (VA)",
        "Total (VA)",
        "Circuit Breaker (AT/AF/Poles/kA)",
        "Cable Conductor (FD-CV mm²)",
        "Ground Wire (IEC01 mm²)",
        "Conduit (HDPE mm)",
        "Length (m)",
        "% Voltage Drop",
    ];

    for (col_idx, &h) in headers.iter().enumerate() {
        worksheet.write_string_with_format(header_row, col_idx as u16, h, &th_format)?;
    }

    // --- ส่วนที่ 3: แถวข้อมูลวงจรย่อย (Circuit Rows) ---
    let mut current_row = header_row + 1;
    let mut ckt_num = 1;

    let mut sum_phase_a = 0.0;
    let mut sum_phase_b = 0.0;
    let mut sum_phase_c = 0.0;

    // 1. ตู้ EV Charger แต่ละตู้
    for circuit in &output.charger_circuits {
        let each_phase_va = circuit.rated_current_a * (output.load_summary.system_voltage_v / 3.0_f64.sqrt());
        let total_va = each_phase_va * 3.0;

        sum_phase_a += each_phase_va;
        sum_phase_b += each_phase_va;
        sum_phase_c += each_phase_va;

        worksheet.write_string_with_format(current_row, 0, format!("{:02}", ckt_num), &cell_center)?;
        worksheet.write_string_with_format(current_row, 1, &circuit.charger_name, &cell_left)?;
        worksheet.write_number_with_format(current_row, 2, each_phase_va, &cell_number)?;
        worksheet.write_number_with_format(current_row, 3, each_phase_va, &cell_number)?;
        worksheet.write_number_with_format(current_row, 4, each_phase_va, &cell_number)?;
        worksheet.write_number_with_format(current_row, 5, total_va, &cell_number)?;

        worksheet.write_string_with_format(current_row, 6, &circuit.branch_breaker.name, &cell_center)?;
        let phase_cable_desc = if circuit.cable_sizing.cable_spec.sets > 1 {
            format!(
                "{}x(4x1/C-{} mm²)",
                circuit.cable_sizing.cable_spec.sets,
                circuit.cable_sizing.cable_spec.conductor_size_sqmm
            )
        } else {
            format!(
                "4x1/C-{} mm²",
                circuit.cable_sizing.cable_spec.conductor_size_sqmm
            )
        };
        worksheet.write_string_with_format(current_row, 7, &phase_cable_desc, &cell_center)?;
        let ground_cable_desc = format!(
            "1-{} mm²",
            circuit.cable_sizing.cable_spec.ground_size_sqmm
        );
        worksheet.write_string_with_format(current_row, 8, &ground_cable_desc, &cell_center)?;
        let conduit_desc = if circuit.cable_sizing.cable_spec.sets > 1 {
            format!(
                "{}x(HDPE {})",
                circuit.cable_sizing.cable_spec.sets, circuit.cable_sizing.cable_spec.conduit_size_mm
            )
        } else {
            format!("HDPE {}", circuit.cable_sizing.cable_spec.conduit_size_mm)
        };
        worksheet.write_string_with_format(current_row, 9, &conduit_desc, &cell_center)?;
        worksheet.write_number_with_format(current_row, 10, circuit.cable_sizing.length_m, &cell_center)?;
        worksheet.write_number_with_format(
            current_row,
            11,
            circuit.cable_sizing.voltage_drop_percent,
            &cell_decimal,
        )?;

        current_row += 1;
        ckt_num += 1;
    }

    // 2. โหลดเสริม Lighting & Power (LP Panel)
    let lp_va = 5888.0; // อ้างอิงตู้ LP ใน Load Schedule
    let lp_phase_va = lp_va / 3.0;
    sum_phase_a += lp_phase_va;
    sum_phase_b += lp_phase_va;
    sum_phase_c += lp_phase_va;

    worksheet.write_string_with_format(current_row, 0, format!("{:02}", ckt_num), &cell_center)?;
    worksheet.write_string_with_format(
        current_row,
        1,
        "LP Panel (Lighting & Aux Power)",
        &cell_left,
    )?;
    worksheet.write_number_with_format(current_row, 2, lp_phase_va, &cell_number)?;
    worksheet.write_number_with_format(current_row, 3, lp_phase_va, &cell_number)?;
    worksheet.write_number_with_format(current_row, 4, lp_phase_va, &cell_number)?;
    worksheet.write_number_with_format(current_row, 5, lp_va, &cell_number)?;

    worksheet.write_string_with_format(current_row, 6, &output.lp_branch_breaker.name, &cell_center)?;
    worksheet.write_string_with_format(current_row, 7, "4x1/C-6 mm²", &cell_center)?;
    worksheet.write_string_with_format(current_row, 8, "1-6 mm²", &cell_center)?;
    worksheet.write_string_with_format(current_row, 9, "HDPE 32", &cell_center)?;
    worksheet.write_number_with_format(current_row, 10, 15.0, &cell_center)?;
    worksheet.write_number_with_format(current_row, 11, 0.45, &cell_decimal)?;

    current_row += 1;
    ckt_num += 1;

    // 3. วงจรสำรอง (Spare / Space)
    worksheet.write_string_with_format(current_row, 0, format!("{:02}", ckt_num), &cell_center)?;
    worksheet.write_string_with_format(current_row, 1, "SPARE (EV Charger Extension)", &cell_left)?;
    worksheet.write_string_with_format(current_row, 2, "-", &cell_center)?;
    worksheet.write_string_with_format(current_row, 3, "-", &cell_center)?;
    worksheet.write_string_with_format(current_row, 4, "-", &cell_center)?;
    worksheet.write_string_with_format(current_row, 5, "-", &cell_center)?;
    worksheet.write_string_with_format(current_row, 6, "Space for MCCB 400AF", &cell_center)?;
    worksheet.write_string_with_format(current_row, 7, "-", &cell_center)?;
    worksheet.write_string_with_format(current_row, 8, "-", &cell_center)?;
    worksheet.write_string_with_format(current_row, 9, "-", &cell_center)?;
    worksheet.write_string_with_format(current_row, 10, "-", &cell_center)?;
    worksheet.write_string_with_format(current_row, 11, "-", &cell_center)?;

    current_row += 2;

    // --- ส่วนที่ 4: สรุปผลรวม (Summary Rows) ---
    // Total Connected Load
    let grand_total_connected_va = sum_phase_a + sum_phase_b + sum_phase_c;
    worksheet.merge_range(
        current_row,
        0,
        current_row,
        1,
        "TOTAL CONNECTED LOAD (VA)",
        &total_label_format,
    )?;
    worksheet.write_number_with_format(current_row, 2, sum_phase_a, &total_number_format)?;
    worksheet.write_number_with_format(current_row, 3, sum_phase_b, &total_number_format)?;
    worksheet.write_number_with_format(current_row, 4, sum_phase_c, &total_number_format)?;
    worksheet.write_number_with_format(current_row, 5, grand_total_connected_va, &total_number_format)?;
    let total_conn_str = format!("{:.2} kVA", grand_total_connected_va / 1000.0);
    worksheet.write_string_with_format(
        current_row,
        6,
        &total_conn_str,
        &total_label_format,
    )?;

    current_row += 1;

    // Total Demand Load
    let demand_va = grand_total_connected_va * output.load_summary.coincidence_factor;
    let demand_label = format!(
        "TOTAL DEMAND LOAD (DF: {:.2})",
        output.load_summary.coincidence_factor
    );
    worksheet.merge_range(
        current_row,
        0,
        current_row,
        1,
        &demand_label,
        &total_label_format,
    )?;
    worksheet.write_number_with_format(
        current_row,
        2,
        sum_phase_a * output.load_summary.coincidence_factor,
        &total_number_format,
    )?;
    worksheet.write_number_with_format(
        current_row,
        3,
        sum_phase_b * output.load_summary.coincidence_factor,
        &total_number_format,
    )?;
    worksheet.write_number_with_format(
        current_row,
        4,
        sum_phase_c * output.load_summary.coincidence_factor,
        &total_number_format,
    )?;
    worksheet.write_number_with_format(current_row, 5, demand_va, &total_number_format)?;
    let total_demand_str = format!("{:.2} kVA", demand_va / 1000.0);
    worksheet.write_string_with_format(
        current_row,
        6,
        &total_demand_str,
        &total_label_format,
    )?;

    current_row += 1;

    // Max Continuous Operating Current
    worksheet.merge_range(
        current_row,
        0,
        current_row,
        1,
        "MAX OPERATING CURRENT (A)",
        &total_label_format,
    )?;
    let max_current_str = format!("{:.2} A", output.load_summary.max_continuous_current_a);
    worksheet.merge_range(
        current_row,
        2,
        current_row,
        5,
        &max_current_str,
        &total_number_format,
    )?;
    let busbar_str = format!(
        "Busbar Rating: {} AF",
        output.main_breaker.frame_af
    );
    worksheet.write_string_with_format(
        current_row,
        6,
        &busbar_str,
        &total_label_format,
    )?;

    // กำหนดความกว้างคอลัมน์อัตโนมัติให้สวยงาม
    worksheet.set_column_width(0, 10)?;
    worksheet.set_column_width(1, 32)?;
    worksheet.set_column_width(2, 16)?;
    worksheet.set_column_width(3, 16)?;
    worksheet.set_column_width(4, 16)?;
    worksheet.set_column_width(5, 18)?;
    worksheet.set_column_width(6, 32)?;
    worksheet.set_column_width(7, 24)?;
    worksheet.set_column_width(8, 20)?;
    worksheet.set_column_width(9, 18)?;
    worksheet.set_column_width(10, 12)?;
    worksheet.set_column_width(11, 16)?;

    let buffer = workbook.save_to_buffer()?;
    Ok(buffer)
}

/// ส่งออกตาราง Load Schedule เป็นไฟล์ Excel (.xlsx) ไปยังพาธที่กำหนด
pub fn export_load_schedule(
    output: &StationOutput,
    file_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = export_load_schedule_bytes(output)?;
    std::fs::write(file_path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuxLoad, ChargerType, ChargerUnit, PhaseConnection, StationDesignEngine, StationInput};

    #[test]
    fn test_export_load_schedule_creates_valid_excel() {
        let charger = ChargerUnit {
            id: "EV-01".to_string(),
            name: "DC Fast Charger 180kW".to_string(),
            charger_type: ChargerType::DcFast(180),
            power_kw: 180.0,
            distance_m: 20.0,
            power_factor: 1.0,
            efficiency: 1.0,
        };

        let aux_load = AuxLoad::new("LP Panel", 5888.0, PhaseConnection::ThreePhase);

        let input = StationInput {
            chargers: vec![charger],
            aux_loads: vec![aux_load],
            coincidence_factor: 1.0,
            system_voltage_v: Some(400.0),
            max_voltage_drop_percent: Some(2.0),
            main_cable_distance_m: Some(20.0),
        };

        let engine = StationDesignEngine::new();
        let output = engine.calculate_station(&input).expect("Calculation failed");

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_load_schedule.xlsx");

        let res = export_load_schedule(&output, &test_file);
        assert!(res.is_ok(), "Excel export should succeed: {:?}", res.err());
        assert!(test_file.exists(), "Exported Excel file should exist");

        // ทำความสะอาดไฟล์ทดสอบ
        let _ = std::fs::remove_file(test_file);
    }
}
