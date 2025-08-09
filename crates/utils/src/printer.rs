use comfy_table::{Table, Row, Cell, presets::UTF8_FULL, ContentArrangement, Color};

pub struct MatrixRow {
    pub profitable: bool,
    pub path: String,
    pub start: f64,
    pub end: f64,
    pub delta_bps: f64, // 原始 bps（基点）
}

pub fn print_matrix_table(rows: Vec<MatrixRow>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Arbitrage", "Path", "Start", "End", "Change (%)"]); // 改成人话百分比

    for row in rows {
        // bps -> 百分比（%）
        let pct = row.delta_bps / 100.0;
        let pct_str = format!("{:+.4}%", pct);

        // 盈利绿色，亏损红色
        let pct_cell = if row.profitable {
            Cell::new(pct_str).fg(Color::Green)
        } else {
            Cell::new(pct_str).fg(Color::Red)
        };

        table.add_row(Row::from(vec![
            Cell::new(if row.profitable { "✅ Profitable" } else { "🧊 No Profit" }),
            Cell::new(row.path),
            Cell::new(format!("{:.6}", row.start)),
            Cell::new(format!("{:.6}", row.end)),
            pct_cell,
        ]));
    }

    println!("{table}");
}
