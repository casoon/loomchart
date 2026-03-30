//! Report generation for backtest results.

use crate::metrics::{Metrics, EquityPoint, Trade};
use crate::BacktestResult;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, format};

/// Report generator
pub struct Report {
    result: BacktestResult,
    strategy_name: String,
    strategy_params: Vec<(String, String)>,
}

impl Report {
    pub fn new(result: BacktestResult, strategy_name: &str) -> Self {
        Self {
            result,
            strategy_name: strategy_name.into(),
            strategy_params: Vec::new(),
        }
    }

    pub fn with_params(mut self, params: Vec<(String, String)>) -> Self {
        self.strategy_params = params;
        self
    }

    /// Generate full text report
    pub fn to_text(&self) -> String {
        let mut s = String::new();

        s.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        s.push_str("║              BACKTEST REPORT                                 ║\n");
        s.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

        // Strategy info
        s.push_str(&format!("Strategy: {}\n", self.strategy_name));
        if !self.strategy_params.is_empty() {
            s.push_str("Parameters:\n");
            for (key, value) in &self.strategy_params {
                s.push_str(&format!("  {} = {}\n", key, value));
            }
        }
        s.push_str("\n");

        // Performance
        s.push_str("─────────────────────────────────────────────────────────────────\n");
        s.push_str("                     PERFORMANCE SUMMARY\n");
        s.push_str("─────────────────────────────────────────────────────────────────\n\n");

        let m = &self.result.metrics;

        s.push_str(&format!("Total Return:        {:>12.2} ({:>+.2}%)\n",
            m.total_return, m.total_return_pct));
        s.push_str(&format!("Annual Return:       {:>12.2}%\n", m.annualized_return));
        s.push_str(&format!("Final Equity:        {:>12.2}\n", self.result.final_equity));
        s.push_str("\n");

        // Risk metrics
        s.push_str("─────────────────────────────────────────────────────────────────\n");
        s.push_str("                       RISK METRICS\n");
        s.push_str("─────────────────────────────────────────────────────────────────\n\n");

        s.push_str(&format!("Volatility:          {:>12.2}%\n", m.volatility * 100.0));
        s.push_str(&format!("Max Drawdown:        {:>12.2} ({:.2}%)\n",
            m.max_drawdown, m.max_drawdown_pct));
        s.push_str(&format!("Sharpe Ratio:        {:>12.2}\n", m.sharpe_ratio));
        s.push_str(&format!("Sortino Ratio:       {:>12.2}\n", m.sortino_ratio));
        s.push_str(&format!("Calmar Ratio:        {:>12.2}\n", m.calmar_ratio));
        s.push_str("\n");

        // Trade statistics
        s.push_str("─────────────────────────────────────────────────────────────────\n");
        s.push_str("                     TRADE STATISTICS\n");
        s.push_str("─────────────────────────────────────────────────────────────────\n\n");

        s.push_str(&format!("Total Trades:        {:>12}\n", m.total_trades));
        s.push_str(&format!("Winning Trades:      {:>12}\n", m.winning_trades));
        s.push_str(&format!("Losing Trades:       {:>12}\n", m.losing_trades));
        s.push_str(&format!("Win Rate:            {:>12.1}%\n", m.win_rate));
        s.push_str(&format!("Profit Factor:       {:>12.2}\n", m.profit_factor));
        s.push_str("\n");
        s.push_str(&format!("Average Trade:       {:>12.2}\n", m.average_trade));
        s.push_str(&format!("Average Win:         {:>12.2}\n", m.average_win));
        s.push_str(&format!("Average Loss:        {:>12.2}\n", m.average_loss));
        s.push_str(&format!("Largest Win:         {:>12.2}\n", m.largest_win));
        s.push_str(&format!("Largest Loss:        {:>12.2}\n", m.largest_loss));
        s.push_str("\n");
        s.push_str(&format!("Max Consec Wins:     {:>12}\n", m.max_consecutive_wins));
        s.push_str(&format!("Max Consec Losses:   {:>12}\n", m.max_consecutive_losses));
        s.push_str("\n");

        // Costs
        s.push_str("─────────────────────────────────────────────────────────────────\n");
        s.push_str("                         COSTS\n");
        s.push_str("─────────────────────────────────────────────────────────────────\n\n");

        s.push_str(&format!("Total Commission:    {:>12.2}\n", m.total_commission));
        s.push_str(&format!("Total Slippage:      {:>12.2}\n", m.total_slippage));
        s.push_str(&format!("Time in Market:      {:>12.1}%\n", m.time_in_market));
        s.push_str("\n");

        // Trade list (last 10)
        if !self.result.trades.is_empty() {
            s.push_str("─────────────────────────────────────────────────────────────────\n");
            s.push_str("                     RECENT TRADES\n");
            s.push_str("─────────────────────────────────────────────────────────────────\n\n");

            s.push_str("  #  | Side  | Qty    | Entry     | Exit      | PnL       | Ret%\n");
            s.push_str("-----|-------|--------|-----------|-----------|-----------|------\n");

            for (i, trade) in self.result.trades.iter().rev().take(10).enumerate() {
                s.push_str(&format!(
                    "{:>4} | {:>5} | {:>6.2} | {:>9.2} | {:>9.2} | {:>+9.2} | {:>+5.1}%\n",
                    self.result.trades.len() - i,
                    trade.side,
                    trade.quantity,
                    trade.entry_price,
                    trade.exit_price,
                    trade.pnl,
                    trade.return_pct
                ));
            }

            if self.result.trades.len() > 10 {
                s.push_str(&format!("\n... and {} more trades\n", self.result.trades.len() - 10));
            }
        }

        s.push_str("\n═══════════════════════════════════════════════════════════════\n");

        s
    }

    /// Generate CSV of trades
    pub fn trades_to_csv(&self) -> String {
        let mut s = String::new();
        s.push_str("symbol,side,quantity,entry_price,exit_price,entry_time,exit_time,pnl,return_pct,commission,slippage\n");

        for trade in &self.result.trades {
            s.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{}\n",
                trade.symbol,
                trade.side,
                trade.quantity,
                trade.entry_price,
                trade.exit_price,
                trade.entry_time,
                trade.exit_time,
                trade.pnl,
                trade.return_pct,
                trade.commission,
                trade.slippage
            ));
        }

        s
    }

    /// Generate CSV of equity curve
    pub fn equity_to_csv(&self) -> String {
        let mut s = String::new();
        s.push_str("timestamp,equity,cash,positions_value\n");

        for point in &self.result.equity_curve {
            s.push_str(&format!(
                "{},{},{},{}\n",
                point.timestamp,
                point.equity,
                point.cash,
                point.positions_value
            ));
        }

        s
    }

    /// Generate JSON report
    #[cfg(feature = "serde")]
    pub fn to_json(&self) -> String {
        #[derive(serde::Serialize)]
        struct JsonReport<'a> {
            strategy_name: &'a str,
            parameters: &'a [(String, String)],
            metrics: &'a Metrics,
            final_equity: f64,
            total_bars: usize,
            trade_count: usize,
        }

        let report = JsonReport {
            strategy_name: &self.strategy_name,
            parameters: &self.strategy_params,
            metrics: &self.result.metrics,
            final_equity: self.result.final_equity,
            total_bars: self.result.total_bars,
            trade_count: self.result.trades.len(),
        };

        serde_json::to_string_pretty(&report).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_result() -> BacktestResult {
        BacktestResult {
            metrics: Metrics {
                total_return: 1000.0,
                total_return_pct: 10.0,
                annualized_return: 25.0,
                sharpe_ratio: 1.5,
                max_drawdown: 500.0,
                max_drawdown_pct: 5.0,
                total_trades: 10,
                winning_trades: 6,
                losing_trades: 4,
                win_rate: 60.0,
                profit_factor: 1.8,
                ..Default::default()
            },
            equity_curve: Vec::new(),
            trades: Vec::new(),
            final_equity: 11000.0,
            total_bars: 1000,
        }
    }

    #[test]
    fn test_text_report() {
        let result = sample_result();
        let report = Report::new(result, "Test Strategy")
            .with_params(vec![("period".into(), "14".into())]);

        let text = report.to_text();
        assert!(text.contains("Test Strategy"));
        assert!(text.contains("10.00%")); // return pct
    }
}
