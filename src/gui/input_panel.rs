use egui::{CollapsingHeader, DragValue, ScrollArea, Ui};

use crate::gui::EvStationApp;
use crate::models::{AuxLoad, ChargerType, ChargerUnit, PhaseConnection};

/// ตัวเลือกขนาดมาตรฐานของตู้ชาร์จ
#[derive(Debug, Clone, Copy, PartialEq)]
enum PresetCharger {
    Dc60,
    Dc120,
    Dc180,
    Dc240,
    Dc360,
    Ac7,
    Ac22,
}

impl PresetCharger {
    fn label(&self) -> &'static str {
        match self {
            PresetCharger::Dc60 => "DC Fast Charger 60 kW",
            PresetCharger::Dc120 => "DC Fast Charger 120 kW",
            PresetCharger::Dc180 => "DC Fast Charger 180 kW",
            PresetCharger::Dc240 => "DC Fast Charger 240 kW",
            PresetCharger::Dc360 => "DC Fast Charger 360 kW",
            PresetCharger::Ac7 => "AC Type 2 7.4 kW (1-Phase)",
            PresetCharger::Ac22 => "AC Type 2 22 kW (3-Phase)",
        }
    }

    fn to_charger_unit(&self, id: String, name: String, distance_m: f64) -> ChargerUnit {
        match self {
            PresetCharger::Dc60 => ChargerUnit {
                id,
                name,
                charger_type: ChargerType::DcFast(60),
                power_kw: 60.0,
                distance_m,
                power_factor: 0.98,
                efficiency: 0.95,
            },
            PresetCharger::Dc120 => ChargerUnit {
                id,
                name,
                charger_type: ChargerType::DcFast(120),
                power_kw: 120.0,
                distance_m,
                power_factor: 0.98,
                efficiency: 0.95,
            },
            PresetCharger::Dc180 => ChargerUnit {
                id,
                name,
                charger_type: ChargerType::DcFast(180),
                power_kw: 180.0,
                distance_m,
                power_factor: 1.0, // ตามแบบ Load Schedule
                efficiency: 1.0,
            },
            PresetCharger::Dc240 => ChargerUnit {
                id,
                name,
                charger_type: ChargerType::DcFast(240),
                power_kw: 240.0,
                distance_m,
                power_factor: 0.98,
                efficiency: 0.95,
            },
            PresetCharger::Dc360 => ChargerUnit {
                id,
                name,
                charger_type: ChargerType::DcFast(360),
                power_kw: 360.0,
                distance_m,
                power_factor: 0.98,
                efficiency: 0.95,
            },
            PresetCharger::Ac7 => ChargerUnit {
                id,
                name,
                charger_type: ChargerType::AcType2(7),
                power_kw: 7.4,
                distance_m,
                power_factor: 0.95,
                efficiency: 0.98,
            },
            PresetCharger::Ac22 => ChargerUnit {
                id,
                name,
                charger_type: ChargerType::AcType2(22),
                power_kw: 22.0,
                distance_m,
                power_factor: 0.95,
                efficiency: 0.98,
            },
        }
    }

    fn from_charger(unit: &ChargerUnit) -> Self {
        match &unit.charger_type {
            ChargerType::DcFast(kw) => match kw {
                60 => PresetCharger::Dc60,
                120 => PresetCharger::Dc120,
                180 => PresetCharger::Dc180,
                240 => PresetCharger::Dc240,
                360 => PresetCharger::Dc360,
                _ => {
                    if (unit.power_kw - 60.0).abs() < 1.0 {
                        PresetCharger::Dc60
                    } else if (unit.power_kw - 120.0).abs() < 1.0 {
                        PresetCharger::Dc120
                    } else if (unit.power_kw - 180.0).abs() < 1.0 {
                        PresetCharger::Dc180
                    } else if (unit.power_kw - 240.0).abs() < 1.0 {
                        PresetCharger::Dc240
                    } else {
                        PresetCharger::Dc360
                    }
                }
            },
            ChargerType::AcType2(kw) => {
                if *kw <= 7 || (unit.power_kw - 7.4).abs() < 1.0 {
                    PresetCharger::Ac7
                } else {
                    PresetCharger::Ac22
                }
            }
        }
    }
}

/// เรนเดอร์ Input Panel ฝั่งซ้ายสำหรับจัดการข้อมูลสถานีไฟฟ้า
pub fn render_input_panel(app: &mut EvStationApp, ui: &mut Ui) {
    ui.heading("⚡ ตั้งค่าระบบไฟฟ้าสถานี (Configuration)");
    ui.add_space(4.0);
    ui.separator();

    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // 1. หมวดจัดการตู้ชาร์จ (EV Chargers)
            render_chargers_section(app, ui);
            ui.add_space(10.0);
            ui.separator();

            // 2. หมวดโหลดระบบเสริม (Auxiliary Loads เช่น LP Panel)
            render_aux_loads_section(app, ui);
            ui.add_space(10.0);
            ui.separator();

            // 3. หมวดการตั้งค่าระบบและสัมประสิทธิ์ (System Parameters)
            render_system_settings_section(app, ui);
        });

    // ทำการคำนวณใหม่แบบ Reactive หากมีการแก้ไขข้อมูล
    app.recalculate_if_dirty();
}

fn render_chargers_section(app: &mut EvStationApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("🚗 รายการตู้ชาร์จ EV").strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("➕ เพิ่มตู้ชาร์จ").clicked() {
                let count = app.chargers.len() + 1;
                let new_unit = PresetCharger::Dc180.to_charger_unit(
                    format!("EV-{:02}", count),
                    format!("DC Fast Charger 180kW #{:02}", count),
                    20.0,
                );
                app.add_charger(new_unit);
            }
        });
    });

    ui.add_space(6.0);

    let mut remove_index = None;
    let total_chargers = app.chargers.len();

    for (idx, charger) in app.chargers.iter_mut().enumerate() {
        let title = format!("[{}] {} ({} kW)", idx + 1, charger.name, charger.power_kw);
        CollapsingHeader::new(title)
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("พิกัดตู้:");
                    let current_preset = PresetCharger::from_charger(charger);
                    let mut selected_preset = current_preset;

                    egui::ComboBox::from_id_salt(format!("charger_preset_{}", idx))
                        .selected_text(current_preset.label())
                        .show_ui(ui, |ui| {
                            for preset in &[
                                PresetCharger::Dc60,
                                PresetCharger::Dc120,
                                PresetCharger::Dc180,
                                PresetCharger::Dc240,
                                PresetCharger::Dc360,
                                PresetCharger::Ac7,
                                PresetCharger::Ac22,
                            ] {
                                ui.selectable_value(&mut selected_preset, *preset, preset.label());
                            }
                        });

                    if selected_preset != current_preset {
                        let updated = selected_preset.to_charger_unit(
                            charger.id.clone(),
                            charger.name.clone(),
                            charger.distance_m,
                        );
                        *charger = updated;
                        app.is_dirty = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("ระยะสายจาก MDB (ม.):");
                    if ui
                        .add(
                            DragValue::new(&mut charger.distance_m)
                                .speed(1.0)
                                .range(5.0..=200.0)
                                .suffix(" ม."),
                        )
                        .changed()
                    {
                        app.is_dirty = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(format!("ID: {}", charger.id));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // ป้องกันไม่ให้ลบจนเหลือ 0 ตู้
                        let delete_btn = ui.add_enabled(total_chargers > 1, egui::Button::new("🗑 ลบตู้นี้"));
                        if delete_btn.clicked() {
                            remove_index = Some(idx);
                        }
                    });
                });
            });
        ui.add_space(4.0);
    }

    if let Some(idx) = remove_index {
        app.remove_charger(idx);
    }
}

fn render_aux_loads_section(app: &mut EvStationApp, ui: &mut Ui) {
    ui.label(egui::RichText::new("💡 โหลดระบบเสริม (Auxiliary Loads - LP)").strong());
    ui.add_space(4.0);

    // ตรวจสอบความยาวของ aux_unit_modes และ aux_power_factors ให้ตรงกับ aux_loads
    while app.aux_unit_modes.len() < app.aux_loads.len() {
        app.aux_unit_modes.push(crate::gui::AuxUnitMode::Va);
    }
    while app.aux_power_factors.len() < app.aux_loads.len() {
        app.aux_power_factors.push(0.85);
    }

    let system_voltage_v = app.system_voltage_v;

    for idx in 0..app.aux_loads.len() {
        ui.group(|ui| {
            let aux_name = app.aux_loads[idx].name.clone();
            ui.label(egui::RichText::new(&aux_name).strong());

            // 1. เลือกระบบเฟส
            ui.horizontal(|ui| {
                ui.label("ระบบเฟส:");
                let current_phase = app.aux_loads[idx].phase;
                let mut selected_phase = current_phase;

                egui::ComboBox::from_id_salt(format!("aux_phase_{}", idx))
                    .selected_text(match selected_phase {
                        PhaseConnection::ThreePhase => "3-Phase (400V)",
                        PhaseConnection::PhaseA => "1-Phase (Phase A 230V)",
                        PhaseConnection::PhaseB => "1-Phase (Phase B 230V)",
                        PhaseConnection::PhaseC => "1-Phase (Phase C 230V)",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected_phase, PhaseConnection::ThreePhase, "3-Phase (400V)");
                        ui.selectable_value(&mut selected_phase, PhaseConnection::PhaseA, "1-Phase (Phase A 230V)");
                        ui.selectable_value(&mut selected_phase, PhaseConnection::PhaseB, "1-Phase (Phase B 230V)");
                        ui.selectable_value(&mut selected_phase, PhaseConnection::PhaseC, "1-Phase (Phase C 230V)");
                    });

                if selected_phase != current_phase {
                    app.aux_loads[idx].phase = selected_phase;
                    app.is_dirty = true;
                }
            });

            // 2. เลือกหน่วย (VA, W, A) และกรอกค่า
            let unit_mode = app.aux_unit_modes[idx];
            let pf = app.aux_power_factors[idx].clamp(0.1, 1.0);
            let phase = app.aux_loads[idx].phase;

            let voltage_for_load = match phase {
                PhaseConnection::ThreePhase => system_voltage_v,
                _ => system_voltage_v / 3.0_f64.sqrt(), // 230V
            };

            // คำนวณค่าตัวเลขปัจจุบันตามหน่วยที่เลือก
            let mut display_value = match unit_mode {
                crate::gui::AuxUnitMode::Va => app.aux_loads[idx].power_va,
                crate::gui::AuxUnitMode::Watts => app.aux_loads[idx].power_va * pf,
                crate::gui::AuxUnitMode::Amps => match phase {
                    PhaseConnection::ThreePhase => {
                        app.aux_loads[idx].power_va / (3.0_f64.sqrt() * voltage_for_load)
                    }
                    _ => app.aux_loads[idx].power_va / voltage_for_load,
                },
            };

            ui.horizontal(|ui| {
                ui.label("เลือกหน่วย:");
                let mut selected_unit = unit_mode;
                egui::ComboBox::from_id_salt(format!("aux_unit_selector_{}", idx))
                    .selected_text(unit_mode.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected_unit, crate::gui::AuxUnitMode::Va, crate::gui::AuxUnitMode::Va.label());
                        ui.selectable_value(&mut selected_unit, crate::gui::AuxUnitMode::Watts, crate::gui::AuxUnitMode::Watts.label());
                        ui.selectable_value(&mut selected_unit, crate::gui::AuxUnitMode::Amps, crate::gui::AuxUnitMode::Amps.label());
                    });

                if selected_unit != unit_mode {
                    app.aux_unit_modes[idx] = selected_unit;
                }
            });

            ui.horizontal(|ui| {
                ui.label("ขนาดโหลด:");
                let (speed, range_max) = match unit_mode {
                    crate::gui::AuxUnitMode::Va => (10.0, 100000.0),
                    crate::gui::AuxUnitMode::Watts => (10.0, 100000.0),
                    crate::gui::AuxUnitMode::Amps => (0.5, 200.0),
                };

                let changed = ui
                    .add(
                        DragValue::new(&mut display_value)
                            .speed(speed)
                            .range(0.0..=range_max)
                            .suffix(unit_mode.suffix()),
                    )
                    .changed();

                if changed {
                    // แปลงค่ากลับเป็น VA เพื่อคำนวณตามมาตรฐานวิศวกรรมไฟฟ้า
                    let calculated_va = match unit_mode {
                        crate::gui::AuxUnitMode::Va => display_value,
                        crate::gui::AuxUnitMode::Watts => display_value / pf,
                        crate::gui::AuxUnitMode::Amps => match phase {
                            PhaseConnection::ThreePhase => {
                                display_value * 3.0_f64.sqrt() * voltage_for_load
                            }
                            _ => display_value * voltage_for_load,
                        },
                    };
                    app.aux_loads[idx].power_va = calculated_va;
                    app.is_dirty = true;
                }
            });

            // ถ้าเลือกหน่วย W ให้มีช่องปรับ Power Factor
            if unit_mode == crate::gui::AuxUnitMode::Watts {
                ui.horizontal(|ui| {
                    ui.label("Power Factor (pf):");
                    if ui
                        .add(
                            DragValue::new(&mut app.aux_power_factors[idx])
                                .speed(0.01)
                                .range(0.5..=1.0),
                        )
                        .changed()
                    {
                        // อัปเดตค่า VA ใหม่เมื่อ pf เปลี่ยน
                        let new_va = display_value / app.aux_power_factors[idx].clamp(0.1, 1.0);
                        app.aux_loads[idx].power_va = new_va;
                        app.is_dirty = true;
                    }
                });
            }

            // แสดงสรุปค่า VA และกระแสเทียบเคียงด้านล่าง
            let current_a = match phase {
                PhaseConnection::ThreePhase => {
                    app.aux_loads[idx].power_va / (3.0_f64.sqrt() * voltage_for_load)
                }
                _ => app.aux_loads[idx].power_va / voltage_for_load,
            };

            ui.label(
                egui::RichText::new(format!(
                    "ℹ เทียบเท่า: {:.1} VA | กระแส: {:.2} A",
                    app.aux_loads[idx].power_va, current_a
                ))
                .weak()
                .small(),
            );
        });
        ui.add_space(4.0);
    }

    if app.aux_loads.is_empty() {
        if ui.button("➕ เพิ่มโหลด LP Panel").clicked() {
            app.add_aux_load(AuxLoad::new("LP Panel", 5888.0, PhaseConnection::ThreePhase));
        }
    }
}

fn render_system_settings_section(app: &mut EvStationApp, ui: &mut Ui) {
    ui.label(egui::RichText::new("⚙ การตั้งค่าระบบไฟฟ้า (System Parameters)").strong());
    ui.add_space(4.0);

    // 1. ความยาวสายเมน MDB
    ui.horizontal(|ui| {
        ui.label("ระยะสายเมน Transformer -> MDB:");
        if ui
            .add(
                DragValue::new(&mut app.main_cable_distance_m)
                    .speed(1.0)
                    .range(5.0..=100.0)
                    .suffix(" ม."),
            )
            .changed()
        {
            app.is_dirty = true;
        }
    });

    // 2. ขีดจำกัด Voltage Drop
    ui.horizontal(|ui| {
        ui.label("เกณฑ์ยอมรับ Voltage Drop สูงสุด:");
        if ui
            .add(
                DragValue::new(&mut app.max_voltage_drop_percent)
                    .speed(0.1)
                    .range(1.0..=5.0)
                    .suffix(" %"),
            )
            .changed()
        {
            app.is_dirty = true;
        }
    });

    // 3. Coincidence / Diversity Factor
    ui.horizontal(|ui| {
        if ui
            .checkbox(&mut app.diversity_enabled, "ใช้ Coincidence / Diversity Factor")
            .changed()
        {
            app.is_dirty = true;
        }
    });

    if app.diversity_enabled {
        ui.horizontal(|ui| {
            ui.label("Coincidence Factor:");
            if ui
                .add(
                    egui::Slider::new(&mut app.coincidence_factor, 0.5..=1.0)
                        .step_by(0.05),
                )
                .changed()
            {
                app.is_dirty = true;
            }
        });
    }

    // 4. สลับโหมด Theme
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("ธีมการแสดงผล:");
        if ui.selectable_label(app.dark_mode, "🌙 Dark").clicked() {
            app.dark_mode = true;
        }
        if ui.selectable_label(!app.dark_mode, "☀️ Light").clicked() {
            app.dark_mode = false;
        }
    });
}
