pub mod calc;
pub mod export;
pub mod models;

pub use calc::*;
pub use export::*;
pub use models::*;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CalculationError {
    #[error("No chargers or auxiliary loads provided")]
    EmptyStationLoads,
    #[error("System voltage must be positive: {0} V")]
    InvalidVoltage(f64),
    #[error("Coincidence factor must be between 0.0 and 1.0: {0}")]
    InvalidCoincidenceFactor(f64),
    #[error("Calculation overflow or sizing failure: {0}")]
    SizingFailure(String),
}

/// ข้อมูล Input สำหรับการคำนวณและออกแบบระบบไฟฟ้าสถานีชาร์จ
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StationInput {
    /// รายการตู้ชาร์จ
    pub chargers: Vec<ChargerUnit>,
    /// รายการโหลดระบบเสริม (Auxiliary Loads เช่น LP Panel)
    pub aux_loads: Vec<AuxLoad>,
    /// สัมประสิทธิ์การใช้งานพร้อมกัน (Coincidence Factor เช่น 1.0)
    pub coincidence_factor: f64,
    /// แรงดันระบบไฟฟ้า 3 เฟส Line-to-Line (V) เช่น 400.0 V
    pub system_voltage_v: Option<f64>,
    /// ขีดจำกัด Voltage Drop สูงสุดที่ยอมรับได้ (%) เช่น 2.0%
    pub max_voltage_drop_percent: Option<f64>,
    /// ระยะสายเมนระหว่างหม้อแปลงถึง MDB (m)
    pub main_cable_distance_m: Option<f64>,
}

impl Default for StationInput {
    fn default() -> Self {
        Self {
            chargers: Vec::new(),
            aux_loads: Vec::new(),
            coincidence_factor: 1.0,
            system_voltage_v: Some(400.0),
            max_voltage_drop_percent: Some(2.0),
            main_cable_distance_m: Some(20.0),
        }
    }
}

/// รายละเอียดการ Sizing วงจรย่อยของตู้ชาร์จแต่ละตู้
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargerCircuitDesign {
    pub charger_id: String,
    pub charger_name: String,
    pub rated_current_a: f64,
    pub branch_breaker: BreakerSpec,
    pub cable_sizing: CableSizingResult,
}

/// ผลลัพธ์การคำนวณและออกแบบระบบไฟฟ้าสถานีชาร์จทั้งระบบ
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StationOutput {
    /// ผลสรุปโหลดรวมของสถานี
    pub load_summary: StationLoadSummary,
    /// ขนาดหม้อแปลงที่คัดเลือก
    pub transformer: TransformerSpec,
    /// พิกัด Main Breaker (MCCB)
    pub main_breaker: BreakerSpec,
    /// สายเมนจากหม้อแปลงเข้า MDB
    pub main_cable: CableSizingResult,
    /// พิกัด Branch Breaker ของตู้ LP
    pub lp_branch_breaker: BreakerSpec,
    /// วงจรย่อยของแต่ละตู้ชาร์จ
    pub charger_circuits: Vec<ChargerCircuitDesign>,
}

/// เครื่องมือคำนวณและออกแบบวิศวกรรมไฟฟ้าสำหรับสถานีอัดประจุไฟฟ้า
#[derive(Debug, Default, Clone, Copy)]
pub struct StationDesignEngine;

impl StationDesignEngine {
    pub fn new() -> Self {
        Self
    }

    /// ทำการคำนวณและคัดเลือกอุปกรณ์ไฟฟ้าทั้งสถานีตามมาตรฐาน วสท./กฟภ.
    pub fn calculate_station(&self, input: &StationInput) -> Result<StationOutput, CalculationError> {
        if input.chargers.is_empty() && input.aux_loads.is_empty() {
            return Err(CalculationError::EmptyStationLoads);
        }

        let voltage_v = input.system_voltage_v.unwrap_or(400.0);
        if voltage_v <= 0.0 {
            return Err(CalculationError::InvalidVoltage(voltage_v));
        }

        if input.coincidence_factor <= 0.0 || input.coincidence_factor > 1.0 {
            return Err(CalculationError::InvalidCoincidenceFactor(
                input.coincidence_factor,
            ));
        }

        let max_vd_percent = input.max_voltage_drop_percent.unwrap_or(2.0);
        let main_distance_m = input.main_cable_distance_m.unwrap_or(20.0);

        // 1. คำนวณ Load Summary รวม
        let load_summary = calculate_total_station_demand_with_voltage(
            &input.chargers,
            &input.aux_loads,
            input.coincidence_factor,
            voltage_v,
        );

        // 2. Transformer Sizing
        let transformer = size_transformer(load_summary.total_demand_kva);

        // 3. Main Breaker Sizing
        let main_breaker = size_main_breaker(load_summary.max_continuous_current_a);

        // 4. Main Cable Sizing (Main feeder จาก Transformer/MDB)
        let main_cable = size_cable_with_vd(
            load_summary.max_continuous_current_a,
            main_distance_m,
            0.98,
            voltage_v,
            max_vd_percent,
        );

        // 5. LP Branch Breaker
        let lp_branch_breaker = size_lp_branch_breaker();

        // 6. Branch Circuits สำหรับตู้ชาร์จแต่ละตู้
        let mut charger_circuits = Vec::new();
        for ch in &input.chargers {
            let rated_i = calculate_charger_unit_current(ch, voltage_v);
            let branch_cb = size_ev_branch_breaker(rated_i);
            let cable = size_cable_with_vd(
                rated_i,
                ch.distance_m,
                ch.power_factor,
                voltage_v,
                max_vd_percent,
            );

            charger_circuits.push(ChargerCircuitDesign {
                charger_id: ch.id.clone(),
                charger_name: ch.name.clone(),
                rated_current_a: rated_i,
                branch_breaker: branch_cb,
                cable_sizing: cable,
            });
        }

        Ok(StationOutput {
            load_summary,
            transformer,
            main_breaker,
            main_cable,
            lp_branch_breaker,
            charger_circuits,
        })
    }
}
