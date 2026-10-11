use egui::{Color32, Grid, RichText, ScrollArea, Ui};

use crate::gui::EvStationApp;
use crate::export::{
    export_load_schedule_bytes, export_sld_dxf_bytes, trigger_download,
};
use crate::StationOutput;

/// เรนเดอร์หน้าจอสรุปผลทางวิศวกรรมไฟฟ้าบน CentralPanel
pub fn render_results_panel(app: &mut EvStationApp, ui: &mut Ui) {
    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            if let Some(err) = &app.error_message {
                ui.group(|ui| {
                    ui.label(
                        RichText::new(format!("⚠️ ข้อผิดพลาดในการคำนวณ: {}", err))
                            .color(Color32::from_rgb(239, 68, 68))
                            .heading(),
                    );
                });
                return;
            }

            if let Some(result) = &app.calculation_result {
                // 1. Header Status Card: หม้อแปลงไฟฟ้าที่แนะนำ
                render_transformer_header_card(result, ui);
                ui.add_space(10.0);

                // 2. แถบปุ่มส่งออกเอกสารวิศวกรรม (Action Toolbar) ด้านบน
                render_export_toolbar(result, &mut app.export_status, ui);
                ui.add_space(14.0);

                // 3. ตารางสรุปโหลดรวม (Summary Cards / Table)
                render_load_summary_section(result, ui);
                ui.add_space(16.0);

                // 4. ตารางอุปกรณ์ MDB เมน
                render_main_equipment_section(result, ui);
                ui.add_space(16.0);

                // 5. ตารางวงจรย่อยและสายไฟตู้ชาร์จแต่ละตู้ (Branch Circuits Table)
                render_branch_circuits_section(result, ui);
                ui.add_space(16.0);

                // 6. แถบปุ่มส่งออกเอกสารวิศวกรรม (Action Toolbar) ด้านล่าง
                render_export_toolbar(result, &mut app.export_status, ui);
                ui.add_space(16.0);
            } else {
                ui.label("กำลังรอข้อมูลการคำนวณ...");
            };
        });
}

fn render_export_toolbar(
    result: &StationOutput,
    export_status: &mut Option<(String, bool)>,
    ui: &mut Ui,
) {
    let result_clone = result.clone();

    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(30, 41, 59))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(51, 65, 85)))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("📁 แถบคำสั่งส่งออก (Export Actions):")
                        .color(Color32::from_rgb(226, 232, 240))
                        .strong(),
                );
                ui.add_space(8.0);

                let xlsx_btn = egui::Button::new(
                    RichText::new("📊 Export Load Schedule (.xlsx)")
                        .color(Color32::from_rgb(240, 253, 244))
                        .size(16.0)
                        .strong(),
                )
                .fill(Color32::from_rgb(22, 101, 52));

                if ui.add(xlsx_btn).clicked() {
                    match export_load_schedule_bytes(&result_clone) {
                        Ok(bytes) => {
                            let mime = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
                            match trigger_download("EV_Station_Load_Schedule.xlsx", &bytes, mime) {
                                Ok(Some(path)) => {
                                    *export_status = Some((
                                        format!("บันทึก/ดาวน์โหลดสำเร็จ: {}", path.display()),
                                        true,
                                    ));
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    *export_status = Some((
                                        format!("เกิดข้อผิดพลาดในการบันทึก: {}", e),
                                        false,
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            *export_status = Some((
                                format!("เกิดข้อผิดพลาดในการสร้างไฟล์ Excel: {}", e),
                                false,
                            ));
                        }
                    }
                }

                ui.add_space(6.0);

                let dxf_btn = egui::Button::new(
                    RichText::new("📐 Export Single-Line Diagram (.dxf)")
                        .color(Color32::from_rgb(240, 249, 255))
                        .size(16.0)
                        .strong(),
                )
                .fill(Color32::from_rgb(3, 105, 161));

                if ui.add(dxf_btn).clicked() {
                    match export_sld_dxf_bytes(&result_clone) {
                        Ok(bytes) => {
                            let mime = "application/dxf";
                            match trigger_download("EV_Station_SLD.dxf", &bytes, mime) {
                                Ok(Some(path)) => {
                                    *export_status = Some((
                                        format!("บันทึก/ดาวน์โหลดสำเร็จ: {}", path.display()),
                                        true,
                                    ));
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    *export_status = Some((
                                        format!("เกิดข้อผิดพลาดในการบันทึก: {}", e),
                                        false,
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            *export_status = Some((
                                format!("เกิดข้อผิดพลาดในการสร้างไฟล์ CAD DXF: {}", e),
                                false,
                            ));
                        }
                    }
                }
            });

            let mut clear_status = false;
            if let Some((msg, is_success)) = export_status.as_ref() {
                let (bg_color, text_color, border_color) = if *is_success {
                    (
                        Color32::from_rgb(20, 83, 45),
                        Color32::from_rgb(220, 252, 231),
                        Color32::from_rgb(34, 197, 94),
                    )
                } else {
                    (
                        Color32::from_rgb(127, 29, 29),
                        Color32::from_rgb(254, 226, 226),
                        Color32::from_rgb(239, 68, 68),
                    )
                };

                ui.add_space(8.0);
                egui::Frame::group(ui.style())
                    .fill(bg_color)
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .inner_margin(egui::Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let icon = if *is_success { "✅ " } else { "❌ " };
                            ui.label(
                                RichText::new(format!("{}{}", icon, msg))
                                    .color(text_color)
                                    .size(15.0)
                                    .strong(),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("✖").clicked() {
                                    clear_status = true;
                                }
                            });
                        });
                    });
            }

            if clear_status {
                *export_status = None;
            }
        });
}

fn render_transformer_header_card(result: &StationOutput, ui: &mut Ui) {
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(20, 83, 45)) // Deep Forest Emerald
        .stroke(egui::Stroke::new(1.5, Color32::from_rgb(34, 197, 94)))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⚡ ขนาดหม้อแปลงที่แนะนำ:")
                        .color(Color32::from_rgb(220, 252, 231))
                        .size(18.0)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!(
                        "{:.0} kVA (22 kV / 400-230 V)",
                        result.transformer.kva
                    ))
                    .color(Color32::WHITE)
                    .size(24.0)
                    .strong(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "พิกัดกระแสหม้อแปลง: {:.1} A",
                            result.transformer.rated_current_a
                        ))
                        .color(Color32::from_rgb(187, 247, 208))
                        .size(16.0),
                    );
                });
            });
        });
}

fn render_load_summary_section(result: &StationOutput, ui: &mut Ui) {
    ui.label(RichText::new("📊 สรุปโหลดและความต้องการพลังงานไฟฟ้า (Load Summary)").heading());
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        // การ์ดที่ 1: Connected Load
        ui.group(|ui| {
            ui.set_width(180.0);
            ui.label(RichText::new("Connected Load").weak());
            ui.label(
                RichText::new(format!("{:.2} kVA", result.load_summary.total_connected_kva))
                    .size(20.0)
                    .strong(),
            );
            ui.label(format!("{:.0} VA", result.load_summary.total_connected_va));
        });

        // การ์ดที่ 2: Demand Load
        ui.group(|ui| {
            ui.set_width(180.0);
            ui.label(RichText::new("Demand Load").weak());
            ui.label(
                RichText::new(format!("{:.2} kVA", result.load_summary.total_demand_kva))
                    .size(20.0)
                    .color(Color32::from_rgb(56, 189, 248))
                    .strong(),
            );
            ui.label(format!("{:.0} VA", result.load_summary.total_demand_va));
        });

        // การ์ดที่ 3: Max Operating Current
        ui.group(|ui| {
            ui.set_width(180.0);
            ui.label(RichText::new("Max Continuous Current").weak());
            ui.label(
                RichText::new(format!("{:.2} A", result.load_summary.max_continuous_current_a))
                    .size(20.0)
                    .color(Color32::from_rgb(250, 204, 21))
                    .strong(),
            );
            ui.label(format!("ระบบแรงดัน {:.0} V", result.load_summary.system_voltage_v));
        });

        // การ์ดที่ 4: Coincidence Factor
        ui.group(|ui| {
            ui.set_width(180.0);
            ui.label(RichText::new("Coincidence Factor").weak());
            ui.label(
                RichText::new(format!("{:.2}", result.load_summary.coincidence_factor))
                    .size(20.0)
                    .strong(),
            );
            ui.label("Diversity Index");
        });
    });
}

fn render_main_equipment_section(result: &StationOutput, ui: &mut Ui) {
    ui.label(RichText::new("🏢 อุปกรณ์ตู้ประธานสวิตช์บอร์ด (Main Distribution Board - MDB)").heading());
    ui.add_space(4.0);

    ui.group(|ui| {
        Grid::new("mdb_equipment_grid")
            .striped(true)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label(RichText::new("อุปกรณ์ / จุดติดตั้ง").strong());
                ui.label(RichText::new("รายละเอียดพิกัดและขนาด").strong());
                ui.label(RichText::new("แรงดันตก (Voltage Drop)").strong());
                ui.end_row();

                // Main Breaker
                ui.label("เมนเบรกเกอร์ (Main Breaker):");
                ui.label(RichText::new(&result.main_breaker.name).color(Color32::from_rgb(56, 189, 248)));
                ui.label("-");
                ui.end_row();

                // Main Feeder Cable
                ui.label("สายป้อนเมน (Main Feeder):");
                ui.label(&result.main_cable.cable_spec.description);

                let vd_color = if result.main_cable.voltage_drop_percent <= 2.0 {
                    Color32::from_rgb(34, 197, 94) // Green
                } else {
                    Color32::from_rgb(239, 68, 68) // Red
                };

                ui.label(
                    RichText::new(format!(
                        "{:.2} V ({:.2}%)",
                        result.main_cable.voltage_drop_v, result.main_cable.voltage_drop_percent
                    ))
                    .color(vd_color)
                    .strong(),
                );
                ui.end_row();

                // LP Branch Breaker
                ui.label("เบรกเกอร์แผง LP (Lighting & Power):");
                ui.label(&result.lp_branch_breaker.name);
                ui.label("-");
                ui.end_row();
            });
    });
}

fn render_branch_circuits_section(result: &StationOutput, ui: &mut Ui) {
    ui.label(RichText::new("🔌 วงจรย่อยและสายป้อนตู้ชาร์จ (EV Charger Branch Circuits)").heading());
    ui.add_space(4.0);

    ui.group(|ui| {
        Grid::new("branch_circuits_table")
            .striped(true)
            .spacing([14.0, 10.0])
            .show(ui, |ui| {
                // Table Headers
                ui.label(RichText::new("ลำดับ").strong());
                ui.label(RichText::new("ตู้ชาร์จ").strong());
                ui.label(RichText::new("กระแสเต็มพิกัด").strong());
                ui.label(RichText::new("เบรกเกอร์ย่อย (Branch Breaker)").strong());
                ui.label(RichText::new("ขนาดสายเฟส / สายนิวทรัล").strong());
                ui.label(RichText::new("สายดิน (G)").strong());
                ui.label(RichText::new("ท่อร้อยสาย").strong());
                ui.label(RichText::new("แรงดันตก (%VD)").strong());
                ui.end_row();

                for (idx, circuit) in result.charger_circuits.iter().enumerate() {
                    ui.label(format!("{}", idx + 1));
                    ui.label(&circuit.charger_name);
                    ui.label(format!("{:.1} A", circuit.rated_current_a));
                    ui.label(RichText::new(&circuit.branch_breaker.name).color(Color32::from_rgb(56, 189, 248)));
                    
                    let phase_desc = if circuit.cable_sizing.cable_spec.sets > 1 {
                        format!(
                            "{}x(4x1/C-{} mm² FD-CV)",
                            circuit.cable_sizing.cable_spec.sets,
                            circuit.cable_sizing.cable_spec.conductor_size_sqmm
                        )
                    } else {
                        format!("4x1/C-{} mm² FD-CV", circuit.cable_sizing.cable_spec.conductor_size_sqmm)
                    };
                    ui.label(phase_desc);
                    ui.label(format!("1-{} mm² IEC01", circuit.cable_sizing.cable_spec.ground_size_sqmm));
                    
                    let conduit_desc = if circuit.cable_sizing.cable_spec.sets > 1 {
                        format!("{}x(HDPE {} mm)", circuit.cable_sizing.cable_spec.sets, circuit.cable_sizing.cable_spec.conduit_size_mm)
                    } else {
                        format!("HDPE {} mm", circuit.cable_sizing.cable_spec.conduit_size_mm)
                    };
                    ui.label(conduit_desc);

                    let vd_color = if circuit.cable_sizing.voltage_drop_percent <= 2.0 {
                        Color32::from_rgb(34, 197, 94)
                    } else {
                        Color32::from_rgb(239, 68, 68)
                    };

                    let upsized_note = if circuit.cable_sizing.upsized_due_to_vd {
                        " (Upsized)"
                    } else {
                        ""
                    };

                    ui.label(
                        RichText::new(format!(
                            "{:.2}%{}",
                            circuit.cable_sizing.voltage_drop_percent, upsized_note
                        ))
                        .color(vd_color)
                        .strong(),
                    );
                    ui.end_row();
                }
            });
    });
}
