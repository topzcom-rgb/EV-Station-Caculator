use crate::models::{AuxLoad, ChargerUnit};
use serde::{Deserialize, Serialize};

/// มาตรฐานแรงดันไฟฟ้า 3 เฟส กฟภ. (PEA standard 400V / Line-to-Line)
pub const DEFAULT_SYSTEM_VOLTAGE_V: f64 = 400.0;

/// ผลสรุปการคำนวณโหลดรวมและพิกัดกระแสของสถานี
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StationLoadSummary {
    /// Connected Load รวม (VA)
    pub total_connected_va: f64,
    /// Connected Load รวม (kVA)
    pub total_connected_kva: f64,
    /// Demand Load รวม (VA)
    pub total_demand_va: f64,
    /// Demand Load รวม (kVA)
    pub total_demand_kva: f64,
    /// กระแสต่อเนื่องสูงสุดในการทำงาน (A)
    pub max_continuous_current_a: f64,
    /// แรงดันระบบที่ใช้ในการคำนวณ (V)
    pub system_voltage_v: f64,
    /// สัมประสิทธิ์การใช้งานพร้อมกัน (Coincidence Factor / Diversity Factor)
    pub coincidence_factor: f64,
}

/// คำนวณกระแสเต็มพิกัด (Full Load Current) ของตู้ชาร์จแต่ละเครื่อง (3 Phase Line Current)
///
/// สูตร:
/// $$I = \frac{P \times 1000}{\sqrt{3} \times V \times \text{pf} \times \eta}$$
///
/// โดยที่:
/// - `power_kw`: กำลังไฟฟ้าขาออกของตู้ชาร์จ (kW)
/// - `voltage_v`: แรงดันไฟฟ้าระบบ 3 เฟส Line-to-Line (V) เช่น 400.0 V
/// - `power_factor`: ค่าตัวประกอบกำลัง (เช่น 0.98 สำหรับ DC Charger)
/// - `efficiency`: ค่าประสิทธิภาพของตู้ (เช่น 0.95 สำหรับ DC Charger)
pub fn calculate_charger_rated_current(
    power_kw: f64,
    voltage_v: f64,
    power_factor: f64,
    efficiency: f64,
) -> f64 {
    let input_power_w = (power_kw * 1000.0) / efficiency;
    let apparent_power_va = input_power_w / power_factor;
    apparent_power_va / (3.0_f64.sqrt() * voltage_v)
}

/// คำนวณกระแสเต็มพิกัดจากข้อมูล `ChargerUnit` โดยใช้แรงดันระบบที่กำหนด
pub fn calculate_charger_unit_current(charger: &ChargerUnit, voltage_v: f64) -> f64 {
    calculate_charger_rated_current(
        charger.power_kw,
        voltage_v,
        charger.power_factor,
        charger.efficiency,
    )
}

/// คำนวณกำลังไฟฟ้าปรากฏขาเข้า (Input Apparent Power: VA) ของตู้ชาร์จ
pub fn calculate_charger_input_va(charger: &ChargerUnit) -> f64 {
    (charger.power_kw * 1000.0) / (charger.efficiency * charger.power_factor)
}

/// คำนวณผลสรุปโหลดรวมของสถานี (Station Demand & Current Summary)
///
/// โดยพิจารณา:
/// - โหลดตู้ชาร์จทั้งหมดรวมกันตาม Coincidence Factor (Demand Factor)
/// - โหลดระบบเสริม (Auxiliary Loads เช่น LP Panel 5,888 VA)
/// - แรงดันระบบไฟฟ้า 400 V (หรือตามที่ระบุ)
pub fn calculate_total_station_demand(
    chargers: &[ChargerUnit],
    aux_loads: &[AuxLoad],
    coincidence_factor: f64,
) -> StationLoadSummary {
    calculate_total_station_demand_with_voltage(
        chargers,
        aux_loads,
        coincidence_factor,
        DEFAULT_SYSTEM_VOLTAGE_V,
    )
}

/// คำนวณผลสรุปโหลดรวมของสถานีโดยสามารถกำหนดแรงดันไฟฟ้า (V) เองได้
pub fn calculate_total_station_demand_with_voltage(
    chargers: &[ChargerUnit],
    aux_loads: &[AuxLoad],
    coincidence_factor: f64,
    voltage_v: f64,
) -> StationLoadSummary {
    // 1. ตู้ชาร์จ: คำนวณ VA ขาเข้ารวม
    let chargers_connected_va: f64 = chargers.iter().map(calculate_charger_input_va).sum();
    let chargers_demand_va = chargers_connected_va * coincidence_factor;

    // 2. Aux Loads: คำนวณ VA รวม (โหลดเสริมทำงานต่อเนื่อง coincidence factor = 1.0)
    let aux_connected_va: f64 = aux_loads.iter().map(|aux| aux.power_va).sum();
    let aux_demand_va = aux_connected_va;

    // 3. รวมค่า VA และ kVA
    let total_connected_va = chargers_connected_va + aux_connected_va;
    let total_demand_va = chargers_demand_va + aux_demand_va;

    let total_connected_kva = total_connected_va / 1000.0;
    let total_demand_kva = total_demand_va / 1000.0;

    // 4. คำนวณกระแสสูงสุดต่อเนื่อง (Max Continuous Operating Current)
    // I = Total Demand VA / (sqrt(3) * V)
    let max_continuous_current_a = total_demand_va / (3.0_f64.sqrt() * voltage_v);

    StationLoadSummary {
        total_connected_va,
        total_connected_kva,
        total_demand_va,
        total_demand_kva,
        max_continuous_current_a,
        system_voltage_v: voltage_v,
        coincidence_factor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChargerType, PhaseConnection};

    #[test]
    fn test_single_dc_180kw_plus_aux_load_schedule() {
        // อ้างอิงกรณีศึกษาตาม Load Schedule.pdf:
        // - ตู้ DC Fast Charger 180 kW (1 ตู้)
        //   ใน Load Schedule วิศวกรรมไฟฟ้า:
        //   Input Rating หรือ Demand ของตู้ 180kW = 180,000 VA (หรือ 180 kVA)
        //   Aux Load (LP Panel) = 5,888 VA (เช่น แสงสว่าง, CCTV, อุปกรณ์สื่อสาร)
        // - Total Demand VA = 180,000 + 5,888 = 185,888 VA (185.89 kVA)
        // - พิกัดกระแสรวมที่แรงดัน 400V 3-Phase:
        //   I = 185,888 / (sqrt(3) * 400) = 185,888 / 692.820323 = 268.306 A (ประมาณ 268.3 A)

        let charger = ChargerUnit {
            id: "DC-01".to_string(),
            name: "DC Fast Charger 180kW".to_string(),
            charger_type: ChargerType::DcFast(180),
            power_kw: 180.0,
            distance_m: 25.0,
            power_factor: 1.0,
            efficiency: 1.0,
        };

        let aux_load = AuxLoad::new("LP Load", 5888.0, PhaseConnection::ThreePhase);

        let summary = calculate_total_station_demand(&[charger], &[aux_load], 1.0);

        // ตรวจสอบค่า Total Demand VA ตรงกับ 185,888 VA
        let expected_demand_va = 185_888.0;
        assert!(
            (summary.total_demand_va - expected_demand_va).abs() < 1e-2,
            "Total Demand VA should be {}, but got {}",
            expected_demand_va,
            summary.total_demand_va
        );

        // ตรวจสอบพิกัดกระแสรวมตรงกับประมาณ 268.3 A (ที่ 400V)
        assert!(
            (summary.max_continuous_current_a - 268.306).abs() < 0.1,
            "Current should be approx 268.3 A, but got {}",
            summary.max_continuous_current_a
        );
    }

    #[test]
    fn test_charger_rated_current_formula_with_pf_and_eff() {
        // ทดสอบฟังก์ชันคำนวณกระแสกรณีคิด Power Factor (0.98) และ Efficiency (0.95)
        // I = (180 * 1000) / (sqrt(3) * 400 * 0.98 * 0.95)
        // I = 180000 / (692.820323 * 0.931) = 180000 / 645.0157 = 279.06 A
        let rated_current = calculate_charger_rated_current(180.0, 400.0, 0.98, 0.95);
        assert!((rated_current - 279.06).abs() < 0.1);
    }
}
