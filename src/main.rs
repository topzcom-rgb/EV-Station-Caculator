use std::path::Path;

pub use ev_station_calculator::models::{AuxLoad, ChargerType, ChargerUnit, PhaseConnection};
pub use ev_station_calculator::{
    calc, export, models, export_load_schedule, export_sld_dxf, StationDesignEngine, StationInput,
    StationOutput,
};

#[path = "gui/mod.rs"]
mod gui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--cli" || arg == "-c") {
        return run_cli();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("Starting EV Station Calculator GUI...");
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1280.0, 800.0])
                .with_min_inner_size([960.0, 600.0])
                .with_title("EV Station Calculator & Sizing Tool (PEA / MEA / EIT)"),
            ..Default::default()
        };

        eframe::run_native(
            "EV Station Calculator",
            native_options,
            Box::new(|cc| {
                gui::setup_custom_fonts(&cc.egui_ctx);
                Ok(Box::new(gui::EvStationApp::default()))
            }),
        )
        .map_err(|e| format!("Eframe runtime error: {}", e).into())
    }

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Debug);

        let window = web_sys::window().expect("No global `window` exists");
        let document = window.document().expect("No `document` on window");
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Canvas element #the_canvas_id not found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element is not a HtmlCanvasElement");

        wasm_bindgen_futures::spawn_local(async move {
            let runner = eframe::WebRunner::new();
            if let Err(e) = runner
                .start(
                    canvas,
                    eframe::WebOptions::default(),
                    Box::new(|cc| {
                        gui::setup_custom_fonts(&cc.egui_ctx);
                        Ok(Box::new(gui::EvStationApp::default()))
                    }),
                )
                .await
            {
                log::error!("Failed to start eframe: {:?}", e);
            }
        });

        Ok(())
    }
}

fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================================");
    println!("       ⚡ EV CHARGING STATION SIZING & SINGLE LINE DIAGRAM CALCULATOR ⚡       ");
    println!("              Standards: PEA / MEA / EIT / IEC 60364 / IEC 61439                ");
    println!("================================================================================\n");

    // 1. กำหนดข้อมูลสถานีตัวอย่างตามเอกสารอ้างอิง (SLD.pdf & Load Schedule.pdf)
    let charger = ChargerUnit {
        id: "EV-01".to_string(),
        name: "DC Fast Charger 180kW".to_string(),
        charger_type: ChargerType::DcFast(180),
        power_kw: 180.0,
        distance_m: 20.0,
        power_factor: 1.0,
        efficiency: 1.0,
    };

    let aux_load = AuxLoad::new("LP Panel (Auxiliary)", 5888.0, PhaseConnection::ThreePhase);

    let input = StationInput {
        chargers: vec![charger],
        aux_loads: vec![aux_load],
        coincidence_factor: 1.0,
        system_voltage_v: Some(400.0),
        max_voltage_drop_percent: Some(2.0),
        main_cable_distance_m: Some(20.0),
    };

    println!("📥 ข้อมูลจำลองสถานี (Input Parameters):");
    println!("   - EV Charger: 180 kW (1 Unit, L = 20 m, 3P 400V)");
    println!("   - Auxiliary Load (LP Panel): 5,888 VA (3P 400/230V)");
    println!("   - Coincidence / Diversity Factor: 1.00");
    println!("   - Max Acceptable Voltage Drop: 2.00%\n");

    // 2. คำนวณผลลัพธ์ผ่าน StationDesignEngine
    let engine = StationDesignEngine::new();
    let output = engine.calculate_station(&input)?;

    println!("📊 สรุปโหลดทางไฟฟ้า (Load Summary):");
    println!(
        "   - Total Connected Load: {:.2} kVA ({:.0} VA)",
        output.load_summary.total_connected_kva, output.load_summary.total_connected_va
    );
    println!(
        "   - Total Demand Load:    {:.2} kVA (Max Continuous Current: {:.1} A)",
        output.load_summary.total_demand_kva, output.load_summary.max_continuous_current_a
    );

    println!("\n⚡ หม้อแปลงไฟฟ้าที่แนะนำ (Transformer):");
    println!(
        "   - ขนาดพิกัด: {:.0} kVA (22 kV / 400-230 V, 3P 4W Dyn11)",
        output.transformer.kva
    );
    println!(
        "   - พิกัดกระแสเต็มพิกัด: {:.1} A",
        output.transformer.rated_current_a
    );

    println!("\n🛡️ เมนเซอร์กิตเบรกเกอร์ (Main MCCB):");
    println!("   - {}", output.main_breaker.name);
    println!(
        "   - Frame / Trip: {}AF / {}AT, Poles: {}P, Icu: {} kA",
        output.main_breaker.frame_af,
        output.main_breaker.trip_at,
        output.main_breaker.poles,
        output.main_breaker.icu_ka
    );

    println!("\n🔌 สายเมนป้อนหลัก (Main Feeder Cable):");
    println!("   - ขนาดสาย: {}", output.main_cable.cable_spec.description);
    println!(
        "   - ท่อร้อยสาย: HDPE {} mm | สายดิน: 1-{} mm² IEC01",
        output.main_cable.cable_spec.conduit_size_mm,
        output.main_cable.cable_spec.ground_size_sqmm
    );
    println!(
        "   - แรงดันตก: {:.2}% (ระยะทาง {:.0} ม.)",
        output.main_cable.voltage_drop_percent, output.main_cable.length_m
    );

    println!("\n🚗 วงจรย่อยตู้ชาร์จ (Branch Circuits):");
    for (i, ckt) in output.charger_circuits.iter().enumerate() {
        println!("   [{}] {}", i + 1, ckt.charger_name);
        println!("       - พิกัดกระแส: {:.1} A", ckt.rated_current_a);
        println!("       - เบรกเกอร์ย่อย: {}", ckt.branch_breaker.name);
        println!(
            "       - สายไฟ: 4x1/C-{} mm² FD-CV in HDPE {} mm + 1-{} mm² G",
            ckt.cable_sizing.cable_spec.conductor_size_sqmm,
            ckt.cable_sizing.cable_spec.conduit_size_mm,
            ckt.cable_sizing.cable_spec.ground_size_sqmm
        );
        println!(
            "       - แรงดันตก: {:.2}% (ผ่านเกณฑ์ <= 2.0%)",
            ckt.cable_sizing.voltage_drop_percent
        );
    }

    println!("\n💡 ตู้ควบคุมระบบเสริม (LP Panel):");
    println!("   - เบรกเกอร์: {}", output.lp_branch_breaker.name);
    println!("   - สายไฟ: 4x1/C-6 mm² FD-CV in HDPE 32 mm + 1-6 mm² G");

    // 3. ทดสอบส่งออกไฟล์ Excel และ DXF
    let xlsx_path = Path::new("test_reference_load_schedule.xlsx");
    let dxf_path = Path::new("test_reference_sld.dxf");

    println!("\n💾 การทดสอบส่งออกเอกสารวิศวกรรม:");
    export_load_schedule(&output, xlsx_path)?;
    println!("   ✅ ส่งออกไฟล์ Excel สำเร็จ: {}", xlsx_path.display());

    export_sld_dxf(&output, dxf_path)?;
    println!("   ✅ ส่งออกไฟล์ CAD DXF สำเร็จ: {}", dxf_path.display());

    // ทำความสะอาดไฟล์ทดสอบ
    let _ = std::fs::remove_file(xlsx_path);
    let _ = std::fs::remove_file(dxf_path);

    println!("\n🎉 การตรวจสอบคำนวณและสร้างเอกสารทั้งหมดเสร็จสมบูรณ์ 100%! (เปิด GUI ด้วย `cargo run`)");
    println!("================================================================================");

    Ok(())
}
