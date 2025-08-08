use comfy_table::{Table, Row, Cell, presets::UTF8_FULL, ContentArrangement};

/// 表格行数据结构
pub struct MatrixRow {
    pub profitable: bool,
    pub path: String,
    pub start: f64,
    pub end: f64,
}

/// 打印 quote-matrix 表格
pub fn print_matrix_table(rows: Vec<MatrixRow>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Arbitrage", "Path", "Start", "End"]);

    for row in rows {
        table.add_row(Row::from(vec![
            Cell::new(if row.profitable { "✅ Profitable" } else { "🧊 No Profit" }),
            Cell::new(row.path),
            Cell::new(format!("{:.6}", row.start)),
            Cell::new(format!("{:.6}", row.end)),
        ]));
    }

    println!("{}", table);
}
