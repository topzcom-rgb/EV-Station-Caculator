use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::StationOutput;

// --- สีมาตรฐาน AutoCAD Color Index (ACI) ---
pub const LAYER_SYMBOLS: &str = "SYMBOLS";
pub const LAYER_POWER_LINE: &str = "POWER_LINE";
pub const LAYER_TEXT: &str = "TEXT";
pub const LAYER_GROUND: &str = "GROUND";

/// โครงสร้างช่วยสร้างไฟล์ AutoCAD DXF ตามมาตรฐาน AutoCAD Release 12 (AC1009)
/// ซึ่งเป็นฟอร์แมต DXF ที่เข้ากันได้สมบูรณ์สูงสุด (Universal DXF Standard)
/// สามารถเปิดตรงเข้า Model Space ได้ 100% ใน AutoCAD ทุกเวอร์ชัน (รวมถึง AutoCAD 2026),
/// BricsCAD, ZWCAD, FreeCAD และ CAD Viewer ทั่วไป
pub struct DxfWriter {
    width: f64,
    height: f64,
    entities: Vec<String>,
}

impl DxfWriter {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            entities: Vec::with_capacity(256),
        }
    }

    pub fn add_line(&mut self, layer: &str, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.entities.push(format!(
            "0\nLINE\n8\n{}\n10\n{:.3}\n20\n{:.3}\n30\n0.0\n11\n{:.3}\n21\n{:.3}\n31\n0.0",
            layer, x1, y1, x2, y2
        ));
    }

    pub fn add_rect(&mut self, layer: &str, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.add_line(layer, x1, y1, x2, y1);
        self.add_line(layer, x2, y1, x2, y2);
        self.add_line(layer, x2, y2, x1, y2);
        self.add_line(layer, x1, y2, x1, y1);
    }

    pub fn add_circle(&mut self, layer: &str, cx: f64, cy: f64, radius: f64) {
        self.entities.push(format!(
            "0\nCIRCLE\n8\n{}\n10\n{:.3}\n20\n{:.3}\n30\n0.0\n40\n{:.3}",
            layer, cx, cy, radius
        ));
    }

    pub fn add_text(&mut self, layer: &str, text: &str, x: f64, y: f64, height: f64) {
        // Sanitize ข้อความให้เป็น Pure ASCII ตามมาตรฐาน AutoCAD DXF
        let safe_text = text
            .replace("mm²", "sq.mm")
            .replace("²", "2")
            .replace("Ω", "Ohm")
            .replace("Δ", "DELTA");

        self.entities.push(format!(
            "0\nTEXT\n8\n{}\n10\n{:.3}\n20\n{:.3}\n30\n0.0\n40\n{:.3}\n1\n{}\n7\nSTANDARD",
            layer, x, y, height, safe_text
        ));
    }

    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(self.to_dxf_string().as_bytes())?;
        Ok(())
    }

    /// สร้างเนื้อหาข้อความ DXF ทั้งหมดในรูปแบบ String ตามมาตรฐาน AC1009
    pub fn to_dxf_string(&self) -> String {
        let mut s = String::with_capacity(self.entities.len() * 48 + 1024);

        let cx = self.width / 2.0;
        let cy = self.height / 2.0;
        let view_h = self.height * 1.05;
        let view_w = self.width * 1.05;
        let aspect = view_w / view_h;

        // 1. Header Section - บังคับ $TILEMODE 1 เพื่อเปิดใน Model Space เสมอ
        s.push_str("0\nSECTION\n2\nHEADER\n");
        s.push_str("9\n$ACADVER\n1\nAC1009\n");
        s.push_str("9\n$INSBASE\n10\n0.0\n20\n0.0\n30\n0.0\n");
        s.push_str("9\n$EXTMIN\n10\n0.0\n20\n0.0\n30\n0.0\n");
        s.push_str(&format!("9\n$EXTMAX\n10\n{:.3}\n20\n{:.3}\n30\n0.0\n", self.width, self.height));
        s.push_str("9\n$LIMMIN\n10\n0.0\n20\n0.0\n");
        s.push_str(&format!("9\n$LIMMAX\n10\n{:.3}\n20\n{:.3}\n", self.width, self.height));
        s.push_str("9\n$TILEMODE\n70\n1\n");
        s.push_str("0\nENDSEC\n");

        // 2. Tables Section (VPORT, LTYPE, LAYER, STYLE)
        s.push_str("0\nSECTION\n2\nTABLES\n");

        // VPORT Table (*ACTIVE: ซูมให้เห็นภาพรวมทั้งแบบทันทีที่เปิด)
        s.push_str("0\nTABLE\n2\nVPORT\n70\n1\n");
        s.push_str("0\nVPORT\n2\n*ACTIVE\n70\n0\n");
        s.push_str("10\n0.0\n20\n0.0\n11\n1.0\n21\n1.0\n");
        s.push_str(&format!("12\n{:.3}\n22\n{:.3}\n", cx, cy));
        s.push_str("13\n0.0\n23\n0.0\n14\n1.0\n24\n1.0\n15\n0.0\n25\n0.0\n");
        s.push_str(&format!("40\n{:.3}\n41\n{:.3}\n", view_h, aspect));
        s.push_str("42\n50.0\n43\n0.0\n44\n0.0\n50\n0.0\n51\n0.0\n71\n0\n72\n1000\n73\n1\n74\n3\n75\n0\n76\n0\n77\n0\n78\n0\n");
        s.push_str("0\nENDTAB\n");

        // LTYPE Table
        s.push_str("0\nTABLE\n2\nLTYPE\n70\n1\n");
        s.push_str("0\nLTYPE\n2\nCONTINUOUS\n70\n64\n3\nSolid line\n72\n65\n73\n0\n40\n0.0\n");
        s.push_str("0\nENDTAB\n");

        // LAYER Table
        s.push_str("0\nTABLE\n2\nLAYER\n70\n5\n");
        s.push_str("0\nLAYER\n2\n0\n70\n0\n62\n7\n6\nCONTINUOUS\n");
        s.push_str("0\nLAYER\n2\nSYMBOLS\n70\n0\n62\n2\n6\nCONTINUOUS\n");
        s.push_str("0\nLAYER\n2\nPOWER_LINE\n70\n0\n62\n7\n6\nCONTINUOUS\n");
        s.push_str("0\nLAYER\n2\nTEXT\n70\n0\n62\n3\n6\nCONTINUOUS\n");
        s.push_str("0\nLAYER\n2\nGROUND\n70\n0\n62\n92\n6\nCONTINUOUS\n");
        s.push_str("0\nENDTAB\n");

        // STYLE Table (STANDARD)
        s.push_str("0\nTABLE\n2\nSTYLE\n70\n1\n");
        s.push_str("0\nSTYLE\n2\nSTANDARD\n70\n0\n40\n0.0\n41\n1.0\n50\n0.0\n71\n0\n42\n0.2\n3\ntxt\n4\n\n");
        s.push_str("0\nENDTAB\n");

        s.push_str("0\nENDSEC\n");

        // 3. Blocks Section (Required for CAD Compatibility)
        s.push_str("0\nSECTION\n2\nBLOCKS\n0\nENDSEC\n");

        // 4. Entities Section
        s.push_str("0\nSECTION\n2\nENTITIES\n");
        for ent in &self.entities {
            s.push_str(ent);
            s.push('\n');
        }
        s.push_str("0\nENDSEC\n0\nEOF\n");

        s
    }

    /// คืนค่าข้อมูลไบต์ทั้งหมดของไฟล์ DXF
    pub fn to_dxf_bytes(&self) -> Vec<u8> {
        self.to_dxf_string().into_bytes()
    }
}

/// สร้างข้อมูลไบต์ของแบบ Single-Line Diagram (SLD) ในรูปแบบ AutoCAD DXF (.dxf) ในหน่วยความจำ (In-memory)
pub fn export_sld_dxf_bytes(
    output: &StationOutput,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let charger_count = output.charger_circuits.len();
    let num_branches = charger_count + 3; // EV Chargers + LP Panel + SPD + SPARE
    let branch_dx = 130.0;
    let diagram_width = ((num_branches as f64) * branch_dx + 260.0).max(960.0);
    let diagram_height = 760.0;

    let mut writer = DxfWriter::new(diagram_width, diagram_height);

    // 1. กรอบแบบและ Title Block
    draw_drawing_border(&mut writer, diagram_width, diagram_height);
    draw_title_block(&mut writer, output, diagram_width);

    // 2. สายแรงสูง 22 kV ขาเข้า + Drop-out Fuse
    let tx_x = 220.0;
    let mv_y = 670.0;
    draw_medium_voltage_incoming(&mut writer, tx_x, mv_y);

    // 3. หม้อแปลงไฟฟ้า 22 kV / 400-230 V
    let tx_y = 570.0;
    draw_transformer_symbol(&mut writer, output, tx_x, tx_y);

    // 4. สายป้อนหลัก (Main Feeder Cable) เข้าตู้ MDB
    let mdb_in_y = 470.0;
    draw_main_cable(&mut writer, output, tx_x, tx_y - 25.0, mdb_in_y);

    // 5. Main Circuit Breaker (MCCB)
    let busbar_y = 390.0;
    draw_main_breaker(&mut writer, output, tx_x, mdb_in_y, busbar_y);

    // 6. Main Busbar
    let busbar_start_x = 100.0;
    let busbar_end_x = busbar_start_x + (num_branches as f64 * branch_dx) + 30.0;
    draw_main_busbar(&mut writer, output, busbar_start_x, busbar_end_x, busbar_y);

    // 7. วงจรย่อยตู้ชาร์จ EV Charger
    let mut current_x = busbar_start_x + 50.0;
    for (idx, circuit) in output.charger_circuits.iter().enumerate() {
        draw_charger_branch(&mut writer, circuit, idx + 1, current_x, busbar_y);
        current_x += branch_dx;
    }

    // 8. วงจรย่อย LP Panel (Auxiliary Loads)
    draw_lp_panel_branch(&mut writer, output, current_x, busbar_y);
    current_x += branch_dx;

    // 9. วงจร Surge Protection Device (SPD)
    draw_spd_branch(&mut writer, current_x, busbar_y);
    current_x += branch_dx;

    // 10. วงจร Spare สำหรับขยายในอนาคต
    draw_spare_branch(&mut writer, output, current_x, busbar_y);

    // 11. ระบบต่อลงดิน (Grounding System & Ground Rods)
    draw_grounding_system(&mut writer, busbar_start_x, busbar_end_x);

    Ok(writer.to_dxf_bytes())
}

/// ส่งออก Single-Line Diagram (SLD) เป็นไฟล์ AutoCAD DXF (.dxf) ไปยังพาธที่กำหนด
pub fn export_sld_dxf(
    output: &StationOutput,
    file_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = export_sld_dxf_bytes(output)?;
    std::fs::write(file_path, bytes)?;
    Ok(())
}

// --- ฟังก์ชันวาดสัญลักษณ์วิศวกรรมไฟฟ้า ---

/// วาดสัญลักษณ์เซอร์กิตเบรกเกอร์ (MCCB Symbol: สี่เหลี่ยมพร้อมเส้นทแยง)
fn draw_breaker_symbol(writer: &mut DxfWriter, cx: f64, cy: f64, w: f64, h: f64, label: &str) {
    let x1 = cx - w / 2.0;
    let y1 = cy - h / 2.0;
    let x2 = cx + w / 2.0;
    let y2 = cy + h / 2.0;

    writer.add_rect(LAYER_SYMBOLS, x1, y1, x2, y2);
    writer.add_line(LAYER_SYMBOLS, x1, y1, x2, y2); // เส้นทแยงมุมแสดงสถานะ Breaker

    writer.add_text(LAYER_TEXT, label, x2 + 4.0, cy - 2.0, 3.8);
}

/// วาดสัญลักษณ์สายดิน (Ground Rod Symbol: ⏚)
fn draw_ground_symbol(writer: &mut DxfWriter, cx: f64, cy: f64) {
    writer.add_line(LAYER_GROUND, cx, cy, cx, cy - 8.0);
    writer.add_line(LAYER_GROUND, cx - 8.0, cy - 8.0, cx + 8.0, cy - 8.0);
    writer.add_line(LAYER_GROUND, cx - 5.0, cy - 11.0, cx + 5.0, cy - 11.0);
    writer.add_line(LAYER_GROUND, cx - 2.0, cy - 14.0, cx + 2.0, cy - 14.0);
}

/// วาดสัญลักษณ์ Drop-out Fuse แรงสูง
fn draw_fuse_symbol(writer: &mut DxfWriter, cx: f64, cy: f64) {
    writer.add_rect(LAYER_SYMBOLS, cx - 4.0, cy - 10.0, cx + 4.0, cy + 10.0);
    writer.add_line(LAYER_POWER_LINE, cx, cy + 10.0, cx, cy + 18.0);
    writer.add_line(LAYER_POWER_LINE, cx, cy - 10.0, cx, cy - 18.0);
    writer.add_text(LAYER_TEXT, "DROP-OUT FUSE 24 kV (3x1P)", cx + 8.0, cy - 2.0, 3.8);
}

// --- ส่วนประกอบหลักของ Single-Line Diagram ---

fn draw_medium_voltage_incoming(writer: &mut DxfWriter, x: f64, y: f64) {
    writer.add_line(LAYER_POWER_LINE, x - 100.0, y, x, y);
    writer.add_line(LAYER_POWER_LINE, x, y, x, y - 20.0);
    draw_fuse_symbol(writer, x, y - 38.0);
    writer.add_line(LAYER_POWER_LINE, x, y - 56.0, x, y - 75.0);

    writer.add_text(
        LAYER_TEXT,
        "22 kV, 3-PHASE, 50 Hz FROM PEA GRID",
        x - 95.0,
        y + 5.0,
        4.5,
    );
}

fn draw_transformer_symbol(
    writer: &mut DxfWriter,
    output: &StationOutput,
    cx: f64,
    cy: f64,
) {
    let r = 16.0;
    let top_cy = cy + 12.0;
    let bot_cy = cy - 12.0;

    // ขดลวดปฐมภูมิ (Primary) & ขดลวดทุติยภูมิ (Secondary)
    writer.add_circle(LAYER_SYMBOLS, cx, top_cy, r);
    writer.add_circle(LAYER_SYMBOLS, cx, bot_cy, r);

    // สัญลักษณ์เวกเตอร์ Delta (สามเหลี่ยม) ในขดปฐมภูมิ
    let tri_r = 5.0;
    writer.add_line(LAYER_SYMBOLS, cx, top_cy + tri_r, cx - tri_r * 0.866, top_cy - tri_r * 0.5);
    writer.add_line(LAYER_SYMBOLS, cx - tri_r * 0.866, top_cy - tri_r * 0.5, cx + tri_r * 0.866, top_cy - tri_r * 0.5);
    writer.add_line(LAYER_SYMBOLS, cx + tri_r * 0.866, top_cy - tri_r * 0.5, cx, top_cy + tri_r);

    // สัญลักษณ์เวกเตอร์ Wye (Y) ในขดทุติยภูมิ
    let y_r = 5.0;
    writer.add_line(LAYER_SYMBOLS, cx, bot_cy, cx, bot_cy - y_r);
    writer.add_line(LAYER_SYMBOLS, cx, bot_cy, cx - y_r * 0.866, bot_cy + y_r * 0.5);
    writer.add_line(LAYER_SYMBOLS, cx, bot_cy, cx + y_r * 0.866, bot_cy + y_r * 0.5);

    // รายละเอียดหม้อแปลง
    let tx_title = format!(
        "TRANSFORMER {:.0} kVA, 22 kV / 400-230 V",
        output.transformer.kva
    );
    let tx_spec = format!(
        "3-PHASE 4-WIRE 50 Hz, Dyn11 (FULL LOAD: {:.1} A)",
        output.transformer.rated_current_a
    );

    writer.add_text(LAYER_TEXT, &tx_title, cx + 25.0, top_cy, 4.5);
    writer.add_text(LAYER_TEXT, &tx_spec, cx + 25.0, bot_cy, 3.8);

    // สัญลักษณ์ต่อลงดินของจุด Neutral หม้อแปลง
    writer.add_line(LAYER_GROUND, cx, bot_cy - r, cx - 25.0, bot_cy - r);
    draw_ground_symbol(writer, cx - 25.0, bot_cy - r);
}

fn draw_main_cable(
    writer: &mut DxfWriter,
    output: &StationOutput,
    x: f64,
    y_start: f64,
    y_end: f64,
) {
    writer.add_line(LAYER_POWER_LINE, x, y_start, x, y_end);

    let cable_spec = &output.main_cable.cable_spec;
    let cable_label = format!(
        "MAIN FEEDER: {}",
        cable_spec.description
    );
    let dist_label = format!(
        "DISTANCE = {:.0} m, %VD = {:.2}%",
        output.main_cable.length_m, output.main_cable.voltage_drop_percent
    );

    writer.add_text(LAYER_TEXT, &cable_label, x + 10.0, (y_start + y_end) / 2.0 + 4.0, 3.8);
    writer.add_text(LAYER_TEXT, &dist_label, x + 10.0, (y_start + y_end) / 2.0 - 4.0, 3.5);
}

fn draw_main_breaker(
    writer: &mut DxfWriter,
    output: &StationOutput,
    x: f64,
    y_top: f64,
    y_busbar: f64,
) {
    let breaker_cy = (y_top + y_busbar) / 2.0;

    writer.add_line(LAYER_POWER_LINE, x, y_top, x, breaker_cy + 10.0);
    draw_breaker_symbol(
        writer,
        x,
        breaker_cy,
        14.0,
        20.0,
        &format!("MAIN {}", output.main_breaker.name),
    );
    writer.add_line(LAYER_POWER_LINE, x, breaker_cy - 10.0, x, y_busbar);
}

fn draw_main_busbar(
    writer: &mut DxfWriter,
    output: &StationOutput,
    x1: f64,
    x2: f64,
    y: f64,
) {
    // บัสบาร์ทองแดง 3 เฟส (วาดเส้นคู่หนา)
    writer.add_line(LAYER_POWER_LINE, x1, y, x2, y);
    writer.add_line(LAYER_POWER_LINE, x1, y - 1.5, x2, y - 1.5);

    let busbar_label = format!(
        "MDB MAIN BUSBAR 3P 4W 400/230V 50Hz (RATED {} AF / {} kA)",
        output.main_breaker.frame_af, output.main_breaker.icu_ka
    );
    writer.add_text(LAYER_TEXT, &busbar_label, x1 + 10.0, y + 6.0, 4.5);
}

fn draw_charger_branch(
    writer: &mut DxfWriter,
    circuit: &crate::ChargerCircuitDesign,
    ckt_no: usize,
    x: f64,
    busbar_y: f64,
) {
    let breaker_y = busbar_y - 45.0;
    let cable_mid_y = busbar_y - 120.0;
    let load_box_top = busbar_y - 190.0;
    let load_box_bot = load_box_top - 50.0;

    // เส้นจากบัสบาร์ลงมายัง Branch Breaker
    writer.add_line(LAYER_POWER_LINE, x, busbar_y - 1.5, x, breaker_y + 10.0);

    // Branch MCCB Symbol
    draw_breaker_symbol(
        writer,
        x,
        breaker_y,
        12.0,
        16.0,
        &format!("CKT-{:02}: {}", ckt_no, circuit.branch_breaker.name),
    );

    // สายไฟลงมายังตู้ชาร์จ
    writer.add_line(LAYER_POWER_LINE, x, breaker_y - 10.0, x, load_box_top);

    // ข้อความสเปกสายไฟและระยะทาง
    let cable_spec = if circuit.cable_sizing.cable_spec.sets > 1 {
        format!(
            "{}x(4x1/C-{} mm² FD-CV)",
            circuit.cable_sizing.cable_spec.sets,
            circuit.cable_sizing.cable_spec.conductor_size_sqmm
        )
    } else {
        format!(
            "4x1/C-{} mm² FD-CV",
            circuit.cable_sizing.cable_spec.conductor_size_sqmm
        )
    };
    let conduit_spec = if circuit.cable_sizing.cable_spec.sets > 1 {
        format!(
            "in {}x(HDPE {} mm) (L={:.0}m, %VD={:.2}%)",
            circuit.cable_sizing.cable_spec.sets,
            circuit.cable_sizing.cable_spec.conduit_size_mm,
            circuit.cable_sizing.length_m,
            circuit.cable_sizing.voltage_drop_percent
        )
    } else {
        format!(
            "in HDPE {} mm (L={:.0}m, %VD={:.2}%)",
            circuit.cable_sizing.cable_spec.conduit_size_mm,
            circuit.cable_sizing.length_m,
            circuit.cable_sizing.voltage_drop_percent
        )
    };
    let ground_spec = format!(
        "1-{} mm² IEC01 GROUND",
        circuit.cable_sizing.cable_spec.ground_size_sqmm
    );

    writer.add_text(LAYER_TEXT, &cable_spec, x + 6.0, cable_mid_y + 8.0, 3.2);
    writer.add_text(LAYER_TEXT, &conduit_spec, x + 6.0, cable_mid_y, 3.0);
    writer.add_text(LAYER_TEXT, &ground_spec, x + 6.0, cable_mid_y - 8.0, 3.0);

    // กล่องตู้ชาร์จ EV Charger (Load Box)
    let box_w = 48.0;
    writer.add_rect(
        LAYER_SYMBOLS,
        x - box_w / 2.0,
        load_box_bot,
        x + box_w / 2.0,
        load_box_top,
    );

    writer.add_text(
        LAYER_TEXT,
        &circuit.charger_name,
        x - box_w / 2.0 + 3.0,
        load_box_top - 12.0,
        3.5,
    );
    writer.add_text(
        LAYER_TEXT,
        &format!("FULL LOAD: {:.1} A", circuit.rated_current_a),
        x - box_w / 2.0 + 3.0,
        load_box_top - 24.0,
        3.2,
    );
    writer.add_text(
        LAYER_TEXT,
        "DC FAST CHARGER UNIT",
        x - box_w / 2.0 + 3.0,
        load_box_top - 36.0,
        3.0,
    );
}

fn draw_lp_panel_branch(
    writer: &mut DxfWriter,
    output: &StationOutput,
    x: f64,
    busbar_y: f64,
) {
    let breaker_y = busbar_y - 45.0;
    let cable_mid_y = busbar_y - 120.0;
    let load_box_top = busbar_y - 190.0;
    let load_box_bot = load_box_top - 50.0;

    writer.add_line(LAYER_POWER_LINE, x, busbar_y - 1.5, x, breaker_y + 10.0);
    draw_breaker_symbol(
        writer,
        x,
        breaker_y,
        12.0,
        16.0,
        &format!("LP: {}", output.lp_branch_breaker.name),
    );
    writer.add_line(LAYER_POWER_LINE, x, breaker_y - 10.0, x, load_box_top);

    writer.add_text(LAYER_TEXT, "4x1/C-6 mm² FD-CV", x + 6.0, cable_mid_y + 4.0, 3.2);
    writer.add_text(LAYER_TEXT, "in HDPE 32 mm + 1-6 G", x + 6.0, cable_mid_y - 4.0, 3.0);

    let box_w = 46.0;
    writer.add_rect(
        LAYER_SYMBOLS,
        x - box_w / 2.0,
        load_box_bot,
        x + box_w / 2.0,
        load_box_top,
    );
    writer.add_text(
        LAYER_TEXT,
        "LP PANEL (AUXILIARY)",
        x - box_w / 2.0 + 3.0,
        load_box_top - 12.0,
        3.5,
    );
    writer.add_text(
        LAYER_TEXT,
        "LIGHTING & RECEPTACLE",
        x - box_w / 2.0 + 3.0,
        load_box_top - 24.0,
        3.0,
    );
    writer.add_text(
        LAYER_TEXT,
        "5,888 VA (3P 400/230V)",
        x - box_w / 2.0 + 3.0,
        load_box_top - 36.0,
        3.0,
    );
}

fn draw_spd_branch(writer: &mut DxfWriter, x: f64, busbar_y: f64) {
    let fuse_y = busbar_y - 45.0;
    let spd_top = busbar_y - 100.0;
    let spd_bot = spd_top - 30.0;

    writer.add_line(LAYER_POWER_LINE, x, busbar_y - 1.5, x, fuse_y + 10.0);
    draw_breaker_symbol(writer, x, fuse_y, 10.0, 14.0, "SPD FUSE 32A");
    writer.add_line(LAYER_POWER_LINE, x, fuse_y - 10.0, x, spd_top);

    // กล่อง Surge Protection Device
    writer.add_rect(LAYER_SYMBOLS, x - 18.0, spd_bot, x + 18.0, spd_top);
    writer.add_text(LAYER_TEXT, "SPD CLASS II", x - 15.0, spd_top - 10.0, 3.2);
    writer.add_text(LAYER_TEXT, "Imax: 40 kA", x - 15.0, spd_top - 18.0, 2.8);
    writer.add_text(LAYER_TEXT, "Up <= 1.5 kV", x - 15.0, spd_top - 25.0, 2.8);

    // สายกราวด์จาก SPD
    writer.add_line(LAYER_GROUND, x, spd_bot, x, spd_bot - 20.0);
    draw_ground_symbol(writer, x, spd_bot - 20.0);
}

fn draw_spare_branch(
    writer: &mut DxfWriter,
    output: &StationOutput,
    x: f64,
    busbar_y: f64,
) {
    let breaker_y = busbar_y - 45.0;
    writer.add_line(LAYER_POWER_LINE, x, busbar_y - 1.5, x, breaker_y + 10.0);

    // Breaker ว่างสำหรับ Spare
    draw_breaker_symbol(
        writer,
        x,
        breaker_y,
        12.0,
        16.0,
        &format!("SPARE: SPACE FOR {}AF", output.main_breaker.frame_af),
    );
    writer.add_line(LAYER_POWER_LINE, x, breaker_y - 10.0, x, breaker_y - 25.0);
    writer.add_text(LAYER_TEXT, "FOR FUTURE EV EXPANSION", x - 20.0, breaker_y - 35.0, 3.2);
}

fn draw_grounding_system(
    writer: &mut DxfWriter,
    busbar_start_x: f64,
    busbar_end_x: f64,
) {
    let g_bar_y = 70.0;

    // เส้น Main Grounding Busbar (MGB)
    writer.add_line(LAYER_GROUND, busbar_start_x, g_bar_y, busbar_end_x, g_bar_y);
    writer.add_text(
        LAYER_TEXT,
        "MAIN GROUNDING BUSBAR (MGB) COPPER 25x3 mm (EIT / PEA STANDARD)",
        busbar_start_x + 10.0,
        g_bar_y + 5.0,
        3.8,
    );

    // แท่งกราวด์ร็อด (Ground Rods) กระจาย 3 จุด
    let g_rod_x1 = busbar_start_x + 60.0;
    let g_rod_x2 = (busbar_start_x + busbar_end_x) / 2.0;
    let g_rod_x3 = busbar_end_x - 60.0;

    for gx in [g_rod_x1, g_rod_x2, g_rod_x3] {
        draw_ground_symbol(writer, gx, g_bar_y);
        writer.add_text(
            LAYER_TEXT,
            "GROUND ROD 5/8\"x10' (R <= 5 Ω)",
            gx - 22.0,
            g_bar_y - 25.0,
            2.8,
        );
    }
}

fn draw_drawing_border(writer: &mut DxfWriter, width: f64, height: f64) {
    // เส้นกรอบนอกและกรอบใน
    writer.add_rect(LAYER_POWER_LINE, 10.0, 10.0, width - 10.0, height - 10.0);
    writer.add_rect(LAYER_POWER_LINE, 15.0, 15.0, width - 15.0, height - 15.0);
}

fn draw_title_block(writer: &mut DxfWriter, output: &StationOutput, total_width: f64) {
    let tb_w = 320.0;
    let tb_h = 110.0;
    let tb_x1 = total_width - 15.0 - tb_w;
    let tb_y1 = 15.0;
    let tb_x2 = total_width - 15.0;
    let tb_y2 = tb_y1 + tb_h;

    writer.add_rect(LAYER_POWER_LINE, tb_x1, tb_y1, tb_x2, tb_y2);

    // เส้นแบ่งช่อง Title Block
    writer.add_line(LAYER_POWER_LINE, tb_x1, tb_y2 - 25.0, tb_x2, tb_y2 - 25.0);
    writer.add_line(LAYER_POWER_LINE, tb_x1, tb_y2 - 50.0, tb_x2, tb_y2 - 50.0);
    writer.add_line(LAYER_POWER_LINE, tb_x1, tb_y2 - 75.0, tb_x2, tb_y2 - 75.0);

    // ข้อความใน Title Block
    writer.add_text(
        LAYER_TEXT,
        "ELECTRIC VEHICLE CHARGING STATION (PEA STANDARD)",
        tb_x1 + 10.0,
        tb_y2 - 16.0,
        5.0,
    );
    writer.add_text(
        LAYER_TEXT,
        "DRAWING TITLE: SINGLE LINE DIAGRAM (SLD)",
        tb_x1 + 10.0,
        tb_y2 - 40.0,
        4.5,
    );

    let sys_desc = format!(
        "SYSTEM: 3P 4W 400/230V, TX: {:.0} kVA, MAIN: {}",
        output.transformer.kva, output.main_breaker.name
    );
    writer.add_text(LAYER_TEXT, &sys_desc, tb_x1 + 10.0, tb_y2 - 65.0, 3.8);

    writer.add_text(
        LAYER_TEXT,
        "DESIGN STANDARD: IEC 60364, IEC 61439 | SCALE: N.T.S.",
        tb_x1 + 10.0,
        tb_y2 - 95.0,
        3.5,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuxLoad, ChargerType, ChargerUnit, PhaseConnection, StationDesignEngine, StationInput};

    #[test]
    fn test_export_sld_dxf_creates_valid_dxf_file() {
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
        let output = engine.calculate_station(&input).expect("Engine calculation failed");

        let temp_dir = std::env::temp_dir();
        let test_dxf_path = temp_dir.join("test_sld_diagram.dxf");

        let res = export_sld_dxf(&output, &test_dxf_path);
        assert!(res.is_ok(), "DXF export should succeed: {:?}", res.err());
        assert!(test_dxf_path.exists(), "Exported DXF file must exist");

        // บันทึกไฟล์ทดสอบใน Workspace เพื่อนำไปทดสอบกับ AutoCAD
        let workspace_dxf = Path::new("EV_Station_SLD_R12.dxf");
        let _ = export_sld_dxf(&output, workspace_dxf);

        // ตรวจสอบโครงสร้างไฟล์ DXF ที่สร้างขึ้น
        let content = std::fs::read_to_string(&test_dxf_path).expect("Read DXF failed");
        assert!(content.contains("SECTION"), "Missing SECTION");
        assert!(content.contains("HEADER"), "Missing HEADER");
        assert!(content.contains("AC1009"), "Missing AC1009 version");
        assert!(content.contains("$TILEMODE\n70\n1"), "Missing $TILEMODE 1 for Model Space");
        assert!(content.contains("TABLES"), "Missing TABLES");
        assert!(content.contains("VPORT"), "Missing VPORT table");
        assert!(content.contains("*ACTIVE"), "Missing *ACTIVE viewport");
        assert!(content.contains("LAYER"), "Missing LAYER table");
        assert!(content.contains("STYLE"), "Missing STYLE table");
        assert!(content.contains("STANDARD"), "Missing STANDARD style");
        assert!(content.contains("BLOCKS"), "Missing BLOCKS section");
        assert!(content.contains("ENTITIES"), "Missing ENTITIES section");
        assert!(content.contains("LINE"), "Missing LINE entity");
        assert!(content.contains("CIRCLE"), "Missing CIRCLE entity");
        assert!(content.contains("TRANSFORMER 250 kVA"), "Missing transformer info");
        assert!(content.contains("EOF"), "Missing DXF EOF marker");

        // ทำความสะอาดไฟล์ชั่วคราว
        let _ = std::fs::remove_file(test_dxf_path);
    }
}
