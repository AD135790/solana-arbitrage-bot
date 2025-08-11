use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ArbRow {
    pub profitable: bool,
    pub path: String,
    pub start: f64,
    pub end: f64,
    pub delta_bps: f64,
}

#[inline]
pub fn amount_from_ui(decimals: u8, ui: f64) -> u64 {
    (ui * 10f64.powi(decimals as i32)).round() as u64
}
