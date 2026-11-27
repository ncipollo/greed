use chrono::Local;
use greed::progress::{
    ChunkProgress, MonthDetail, MonthProgress, Phase, ProgressReporter, SymbolPlan, TickProgress,
};

/// Shows progress on **stderr**, so a command's result (on stdout) can be redirected on its own:
/// `greed backtest cfg.toml > result.txt` captures just the result while progress stays on screen.
pub struct StderrReporter;

impl ProgressReporter for StderrReporter {
    fn phase(&self, phase: Phase) {
        eprintln!(
            "{}",
            format_phase(&phase, &Local::now().format("%H:%M:%S").to_string())
        );
    }

    fn symbol_plan(&self, plan: SymbolPlan) {
        eprintln!("{}", format_plan(&plan));
    }

    fn month_done(&self, month: MonthProgress) {
        eprintln!("{}", format_month(&month));
    }

    fn chunk(&self, chunk: ChunkProgress) {
        eprintln!("{}", format_chunk(&chunk));
    }

    fn tick(&self, tick: TickProgress) {
        eprintln!("{}", format_tick(&tick));
    }
}

fn format_phase(phase: &Phase, clock: &str) -> String {
    format!("[{clock}] {phase}")
}

fn format_plan(plan: &SymbolPlan) -> String {
    if plan.windows == 0 {
        return format!("  {} already complete, skipping", plan.symbol.symbol);
    }
    format!(
        "  Backfilling {} {} … {} ({} missing windows)",
        plan.symbol.symbol,
        plan.first_month.format("%Y-%m"),
        plan.last_month.format("%Y-%m"),
        plan.windows
    )
}

fn format_month(progress: &MonthProgress) -> String {
    let month = progress.month.format("%Y-%m");
    match (&progress.symbol, &progress.detail) {
        (Some(symbol), MonthDetail::Bars(count)) => {
            format!(
                "  {} {month} ✓ {} bars",
                symbol.symbol,
                thousands(*count as u64)
            )
        }
        (_, MonthDetail::Simulated { equity, trades }) => {
            format!("  {month} · equity {} · {trades} trades", money(*equity))
        }
        (None, MonthDetail::Bars(count)) => {
            format!("  {month} ✓ {} bars", thousands(*count as u64))
        }
    }
}

fn format_chunk(chunk: &ChunkProgress) -> String {
    format!(
        "    fetched {} {} … {}: {} bars, {} new",
        chunk.symbol.symbol,
        chunk.window.start.format("%Y-%m-%d"),
        chunk.window.end.format("%Y-%m-%d"),
        thousands(chunk.fetched as u64),
        thousands(chunk.inserted as u64)
    )
}

fn format_tick(tick: &TickProgress) -> String {
    format!(
        "    {} · equity {} · {} trades",
        tick.time.format("%Y-%m-%d %H:%M"),
        money(tick.equity),
        tick.trades
    )
}

fn money(amount: f64) -> String {
    let sign = if amount < 0.0 { "-" } else { "" };
    let cents = (amount.abs() * 100.0).round() as u64;
    format!("{sign}${}.{:02}", thousands(cents / 100), cents % 100)
}

/// `9812` → `"9,812"`.
fn thousands(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};
    use greed::asset::AssetSymbol;

    fn month(year: i32, month: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, 1).unwrap()
    }

    #[test]
    fn thousands_separators() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_000), "1,000");
        assert_eq!(thousands(9_812), "9,812");
        assert_eq!(thousands(1_234_567), "1,234,567");
    }

    #[test]
    fn money_has_separators_and_cents() {
        assert_eq!(money(104_233.1), "$104,233.10");
        assert_eq!(money(0.0), "$0.00");
        assert_eq!(money(-1_500.5), "-$1,500.50");
    }

    #[test]
    fn phases_are_timestamped() {
        assert_eq!(
            format_phase(&Phase::Simulating, "09:30:00"),
            "[09:30:00] Simulating"
        );
    }

    #[test]
    fn plans_show_the_symbol_months_and_window_count() {
        let plan = SymbolPlan {
            symbol: AssetSymbol::new("VTI"),
            first_month: month(2025, 11),
            last_month: month(2026, 11),
            windows: 8,
        };

        assert_eq!(
            format_plan(&plan),
            "  Backfilling VTI 2025-11 … 2026-11 (8 missing windows)"
        );
        assert_eq!(
            format_plan(&SymbolPlan { windows: 0, ..plan }),
            "  VTI already complete, skipping"
        );
    }

    #[test]
    fn backfill_months_show_bar_counts() {
        let line = format_month(&MonthProgress {
            symbol: Some(AssetSymbol::new("VTI")),
            month: month(2026, 1),
            detail: MonthDetail::Bars(9_812),
        });

        assert_eq!(line, "  VTI 2026-01 ✓ 9,812 bars");
    }

    #[test]
    fn simulated_months_show_equity_and_trades() {
        let line = format_month(&MonthProgress {
            symbol: None,
            month: month(2026, 3),
            detail: MonthDetail::Simulated {
                equity: 104_233.1,
                trades: 12,
            },
        });

        assert_eq!(line, "  2026-03 · equity $104,233.10 · 12 trades");
    }

    #[test]
    fn verbose_lines_show_chunks_and_ticks() {
        let chunk = ChunkProgress {
            symbol: AssetSymbol::new("VTI"),
            window: Utc.with_ymd_and_hms(2026, 1, 15, 0, 0, 0).unwrap()
                ..Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap(),
            fetched: 9_812,
            inserted: 9_800,
        };
        let tick = TickProgress {
            time: Utc.with_ymd_and_hms(2026, 3, 4, 14, 35, 0).unwrap(),
            equity: 100_000.0,
            trades: 3,
        };

        assert_eq!(
            format_chunk(&chunk),
            "    fetched VTI 2026-01-15 … 2026-02-01: 9,812 bars, 9,800 new"
        );
        assert_eq!(
            format_tick(&tick),
            "    2026-03-04 14:35 · equity $100,000.00 · 3 trades"
        );
    }
}
