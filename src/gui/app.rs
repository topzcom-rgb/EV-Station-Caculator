use crate::calc::load::DEFAULT_SYSTEM_VOLTAGE_V;
use crate::models::{AuxLoad, ChargerType, ChargerUnit, PhaseConnection};
use crate::{StationDesignEngine, StationInput, StationOutput};

/// หน่วยของโหลดเสริมที่ผู้ใช้เลือกกรอกใน UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuxUnitMode {
    Va,
    Watts,
    Amps,
}

impl AuxUnitMode {
    pub fn label(&self) -> &'static str {
        match self {
            AuxUnitMode::Va => "VA (โวลต์-แอมแปร์)",
            AuxUnitMode::Watts => "W (วัตต์)",
            AuxUnitMode::Amps => "A (แอมแปร์)",
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            AuxUnitMode::Va => " VA",
            AuxUnitMode::Watts => " W",
            AuxUnitMode::Amps => " A",
        }
    }
}

/// โครงสร้าง App State สำหรับจัดเก็บรายการตู้ชาร์จ, โหลดเสริม, และผลลัพธ์การคำนวณแบบ Reactive
pub struct EvStationApp {
    /// รายการตู้ชาร์จในสถานี
    pub chargers: Vec<ChargerUnit>,
    /// รายการโหลดระบบเสริม (Auxiliary Loads เช่น ตู้ LP)
    pub aux_loads: Vec<AuxLoad>,
    /// หน่วยของโหลดเสริมแต่ละรายการที่แสดงใน UI (สอดคล้องกับ aux_loads ตาม index)
    pub aux_unit_modes: Vec<AuxUnitMode>,
    /// ค่า Power Factor สำหรับโหลดเสริมแต่ละรายการ (ค่าเริ่มต้น 0.85 สำหรับแปลง W <-> VA)
    pub aux_power_factors: Vec<f64>,
    /// เปิด/ปิดการคำนวณ Coincidence / Diversity Factor
    pub diversity_enabled: bool,
    /// สัมประสิทธิ์การใช้งานพร้อมกัน (Coincidence Factor)
    pub coincidence_factor: f64,
    /// แรงดันไฟฟ้าระบบ 3 เฟส Line-to-Line (V)
    pub system_voltage_v: f64,
    /// ขีดจำกัด Voltage Drop สูงสุดที่ยอมรับได้ (%)
    pub max_voltage_drop_percent: f64,
    /// ความยาวสายเมนจากหม้อแปลงเข้า MDB (m)
    pub main_cable_distance_m: f64,
    /// ผลลัพธ์การคำนวณทางวิศวกรรมไฟฟ้า
    pub calculation_result: Option<StationOutput>,
    /// ข้อความแจ้งเตือน Error ล่าสุด (ถ้ามี)
    pub error_message: Option<String>,
    /// Flag บ่งบอกว่ามีการแก้ไขข้อมูลที่ต้องคำนวณใหม่
    pub is_dirty: bool,
    /// โหมดการแสดงผล (Dark mode / Light mode)
    pub dark_mode: bool,
}

impl Default for EvStationApp {
    fn default() -> Self {
        // ค่าเริ่มต้นจำลองสถานีตามเอกสารอ้างอิง SLD และ Load Schedule (ตู้ DC 180kW 1 ตู้ + LP Panel 5,888 VA)
        let default_charger = ChargerUnit {
            id: "EV-01".to_string(),
            name: "DC Fast Charger 180kW".to_string(),
            charger_type: ChargerType::DcFast(180),
            power_kw: 180.0,
            distance_m: 20.0,
            power_factor: 1.0,
            efficiency: 1.0,
        };

        let default_aux = AuxLoad::new("LP Panel (Auxiliary)", 5888.0, PhaseConnection::ThreePhase);

        let mut app = Self {
            chargers: vec![default_charger],
            aux_loads: vec![default_aux],
            aux_unit_modes: vec![AuxUnitMode::Va],
            aux_power_factors: vec![0.85],
            diversity_enabled: false,
            coincidence_factor: 1.0,
            system_voltage_v: DEFAULT_SYSTEM_VOLTAGE_V,
            max_voltage_drop_percent: 2.0,
            main_cable_distance_m: 20.0,
            calculation_result: None,
            error_message: None,
            is_dirty: true,
            dark_mode: true,
        };

        // คำนวณผลลัพธ์รอบแรกทันทีเมื่อเริ่มต้น App
        app.recalculate();
        app
    }
}

impl EvStationApp {
    /// สร้างอินสแตนซ์ใหม่พร้อมข้อมูลเริ่มต้น
    pub fn new() -> Self {
        Self::default()
    }

    /// ทำเครื่องหมายว่าข้อมูลถูกเปลี่ยนแปลงและต้องการคำนวณใหม่
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// เรียกใช้ StationDesignEngine เพื่อคำนวณระบบไฟฟ้าใหม่เมื่อ is_dirty เป็นจริง
    pub fn recalculate_if_dirty(&mut self) {
        if self.is_dirty {
            self.recalculate();
        }
    }

    /// ทำการคำนวณระบบไฟฟ้าทั้งหมดด้วย StationDesignEngine
    pub fn recalculate(&mut self) {
        let factor = if self.diversity_enabled {
            self.coincidence_factor.clamp(0.1, 1.0)
        } else {
            1.0
        };

        let input = StationInput {
            chargers: self.chargers.clone(),
            aux_loads: self.aux_loads.clone(),
            coincidence_factor: factor,
            system_voltage_v: Some(self.system_voltage_v),
            max_voltage_drop_percent: Some(self.max_voltage_drop_percent),
            main_cable_distance_m: Some(self.main_cable_distance_m),
        };

        let engine = StationDesignEngine::new();
        match engine.calculate_station(&input) {
            Ok(output) => {
                self.calculation_result = Some(output);
                self.error_message = None;
            }
            Err(err) => {
                self.calculation_result = None;
                self.error_message = Some(err.to_string());
            }
        }

        self.is_dirty = false;
    }

    /// เพิ่มตู้ชาร์จใหม่
    pub fn add_charger(&mut self, charger: ChargerUnit) {
        self.chargers.push(charger);
        self.mark_dirty();
    }

    /// ลบตู้ชาร์จตาม index
    pub fn remove_charger(&mut self, index: usize) {
        if index < self.chargers.len() {
            self.chargers.remove(index);
            self.mark_dirty();
        }
    }

    /// เพิ่มโหลดระบบเสริมใหม่
    pub fn add_aux_load(&mut self, aux: AuxLoad) {
        self.aux_loads.push(aux);
        self.aux_unit_modes.push(AuxUnitMode::Va);
        self.aux_power_factors.push(0.85);
        self.mark_dirty();
    }

    /// ลบโหลดระบบเสริมตาม index
    pub fn remove_aux_load(&mut self, index: usize) {
        if index < self.aux_loads.len() {
            self.aux_loads.remove(index);
            if index < self.aux_unit_modes.len() {
                self.aux_unit_modes.remove(index);
            }
            if index < self.aux_power_factors.len() {
                self.aux_power_factors.remove(index);
            }
            self.mark_dirty();
        }
    }
}

impl eframe::App for EvStationApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // 1. อัปเดตธีม (Dark / Light)
        crate::gui::theme::apply_theme(ui.ctx(), self.dark_mode);

        // 2. Input Panel ฝั่งซ้าย
        egui::Panel::left("station_input_side_panel")
            .default_size(380.0)
            .size_range(320.0..=480.0)
            .resizable(true)
            .show(ui, |ui| {
                crate::gui::input_panel::render_input_panel(self, ui);
            });

        // 3. Results Panel ตรงกลาง
        egui::CentralPanel::default().show(ui, |ui| {
            crate::gui::results_panel::render_results_panel(self, ui);
        });
    }
}
