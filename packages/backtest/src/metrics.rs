//! Performance metrics calculation.

use loom_core::Timestamp;

#[cfg(not(feature = "std"))]
use alloc::{vec::Vec, string::String};

/// Performance metrics for a backtest
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Metrics {
    // Returns
    pub total_return: f64,
    pub total_return_pct: f64,
    pub annualized_return: f64,

    // Risk
    pub volatility: f64,
    pub max_drawdown: f64,
    pub max_drawdown_pct: f64,
    pub max_drawdown_duration: i64, // in milliseconds

    // Risk-adjusted returns
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,

    // Trade statistics
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub average_trade: f64,
    pub average_trade_duration: i64,

    // Exposure
    pub time_in_market: f64, // percentage
    pub max_consecutive_wins: u32,
    pub max_consecutive_losses: u32,

    // Costs
    pub total_commission: f64,
    pub total_slippage: f64,
}

impl Metrics {
    /// Create empty metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate all metrics from equity curve and trades
    pub fn calculate(
        equity_curve: &[EquityPoint],
        trades: &[Trade],
        initial_capital: f64,
        risk_free_rate: f64,
    ) -> Self {
        let mut metrics = Self::new();

        if equity_curve.is_empty() {
            return metrics;
        }

        // Returns
        let final_equity = equity_curve.last().unwrap().equity;
        metrics.total_return = final_equity - initial_capital;
        metrics.total_return_pct = metrics.total_return / initial_capital * 100.0;

        // Calculate returns series
        let returns: Vec<f64> = equity_curve.windows(2)
            .map(|w| (w[1].equity - w[0].equity) / w[0].equity)
            .collect();

        // Volatility (annualized)
        if !returns.is_empty() {
            let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance: f64 = returns.iter()
                .map(|r| (r - mean_return).powi(2))
                .sum::<f64>() / returns.len() as f64;
            metrics.volatility = variance.sqrt() * (252.0_f64).sqrt(); // Annualized

            // Annualized return
            let total_days = (equity_curve.last().unwrap().timestamp - equity_curve.first().unwrap().timestamp)
                as f64 / 86400000.0;
            if total_days > 0.0 {
                metrics.annualized_return =
                    ((final_equity / initial_capital).powf(365.0 / total_days) - 1.0) * 100.0;
            }

            // Sharpe ratio
            if metrics.volatility > 0.0 {
                metrics.sharpe_ratio = (metrics.annualized_return / 100.0 - risk_free_rate) / metrics.volatility;
            }

            // Sortino ratio (using downside deviation)
            let negative_returns: Vec<f64> = returns.iter()
                .filter(|&&r| r < 0.0)
                .copied()
                .collect();
            if !negative_returns.is_empty() {
                let downside_variance: f64 = negative_returns.iter()
                    .map(|r| r.powi(2))
                    .sum::<f64>() / negative_returns.len() as f64;
                let downside_deviation = downside_variance.sqrt() * (252.0_f64).sqrt();
                if downside_deviation > 0.0 {
                    metrics.sortino_ratio = (metrics.annualized_return / 100.0 - risk_free_rate) / downside_deviation;
                }
            }
        }

        // Drawdown analysis
        let mut peak = initial_capital;
        let mut max_dd = 0.0;
        let mut max_dd_pct = 0.0;
        let mut dd_start: Option<Timestamp> = None;
        let mut max_dd_duration: i64 = 0;

        for point in equity_curve {
            if point.equity > peak {
                peak = point.equity;
                if let Some(start) = dd_start {
                    let duration = point.timestamp - start;
                    max_dd_duration = max_dd_duration.max(duration);
                }
                dd_start = None;
            } else {
                let dd = peak - point.equity;
                let dd_pct = dd / peak * 100.0;
                if dd > max_dd {
                    max_dd = dd;
                    max_dd_pct = dd_pct;
                }
                if dd_start.is_none() {
                    dd_start = Some(point.timestamp);
                }
            }
        }

        metrics.max_drawdown = max_dd;
        metrics.max_drawdown_pct = max_dd_pct;
        metrics.max_drawdown_duration = max_dd_duration;

        // Calmar ratio
        if max_dd_pct > 0.0 {
            metrics.calmar_ratio = metrics.annualized_return / max_dd_pct;
        }

        // Trade statistics
        metrics.total_trades = trades.len() as u32;

        let mut gross_profit = 0.0;
        let mut gross_loss = 0.0;
        let mut consecutive_wins = 0u32;
        let mut consecutive_losses = 0u32;
        let mut total_duration: i64 = 0;

        for trade in trades {
            if trade.pnl > 0.0 {
                metrics.winning_trades += 1;
                gross_profit += trade.pnl;
                metrics.largest_win = metrics.largest_win.max(trade.pnl);
                consecutive_wins += 1;
                metrics.max_consecutive_wins = metrics.max_consecutive_wins.max(consecutive_wins);
                consecutive_losses = 0;
            } else if trade.pnl < 0.0 {
                metrics.losing_trades += 1;
                gross_loss += trade.pnl.abs();
                metrics.largest_loss = metrics.largest_loss.max(trade.pnl.abs());
                consecutive_losses += 1;
                metrics.max_consecutive_losses = metrics.max_consecutive_losses.max(consecutive_losses);
                consecutive_wins = 0;
            }

            total_duration += trade.exit_time - trade.entry_time;
            metrics.total_commission += trade.commission;
            metrics.total_slippage += trade.slippage;
        }

        if metrics.total_trades > 0 {
            metrics.win_rate = metrics.winning_trades as f64 / metrics.total_trades as f64 * 100.0;
            metrics.average_trade = metrics.total_return / metrics.total_trades as f64;
            metrics.average_trade_duration = total_duration / metrics.total_trades as i64;
        }

        if metrics.winning_trades > 0 {
            metrics.average_win = gross_profit / metrics.winning_trades as f64;
        }

        if metrics.losing_trades > 0 {
            metrics.average_loss = gross_loss / metrics.losing_trades as f64;
        }

        if gross_loss > 0.0 {
            metrics.profit_factor = gross_profit / gross_loss;
        }

        // Time in market
        if !equity_curve.is_empty() && equity_curve.len() > 1 {
            let total_time = equity_curve.last().unwrap().timestamp - equity_curve.first().unwrap().timestamp;
            if total_time > 0 {
                let time_with_position: i64 = trades.iter()
                    .map(|t| t.exit_time - t.entry_time)
                    .sum();
                metrics.time_in_market = time_with_position as f64 / total_time as f64 * 100.0;
            }
        }

        metrics
    }

    /// Format metrics as readable string
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str("=== Performance Metrics ===\n\n");

        s.push_str(&format!("Total Return:      ${:.2} ({:.2}%)\n", self.total_return, self.total_return_pct));
        s.push_str(&format!("Annualized Return: {:.2}%\n", self.annualized_return));
        s.push_str(&format!("Volatility:        {:.2}%\n\n", self.volatility * 100.0));

        s.push_str(&format!("Sharpe Ratio:      {:.2}\n", self.sharpe_ratio));
        s.push_str(&format!("Sortino Ratio:     {:.2}\n", self.sortino_ratio));
        s.push_str(&format!("Calmar Ratio:      {:.2}\n\n", self.calmar_ratio));

        s.push_str(&format!("Max Drawdown:      ${:.2} ({:.2}%)\n", self.max_drawdown, self.max_drawdown_pct));
        s.push_str(&format!("DD Duration:       {} days\n\n", self.max_drawdown_duration / 86400000));

        s.push_str(&format!("Total Trades:      {}\n", self.total_trades));
        s.push_str(&format!("Win Rate:          {:.1}%\n", self.win_rate));
        s.push_str(&format!("Profit Factor:     {:.2}\n", self.profit_factor));
        s.push_str(&format!("Average Trade:     ${:.2}\n", self.average_trade));
        s.push_str(&format!("Average Win:       ${:.2}\n", self.average_win));
        s.push_str(&format!("Average Loss:      ${:.2}\n", self.average_loss));
        s.push_str(&format!("Largest Win:       ${:.2}\n", self.largest_win));
        s.push_str(&format!("Largest Loss:      ${:.2}\n\n", self.largest_loss));

        s.push_str(&format!("Time in Market:    {:.1}%\n", self.time_in_market));
        s.push_str(&format!("Max Consec Wins:   {}\n", self.max_consecutive_wins));
        s.push_str(&format!("Max Consec Losses: {}\n\n", self.max_consecutive_losses));

        s.push_str(&format!("Total Commission:  ${:.2}\n", self.total_commission));
        s.push_str(&format!("Total Slippage:    ${:.2}\n", self.total_slippage));

        s
    }
}

/// Point in equity curve
#[derive(Debug, Clone)]
pub struct EquityPoint {
    pub timestamp: Timestamp,
    pub equity: f64,
    pub cash: f64,
    pub positions_value: f64,
}

/// A completed trade
#[derive(Debug, Clone)]
pub struct Trade {
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub entry_time: Timestamp,
    pub exit_time: Timestamp,
    pub pnl: f64,
    pub return_pct: f64,
    pub commission: f64,
    pub slippage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_calculation() {
        let equity_curve = vec![
            EquityPoint { timestamp: 0, equity: 10000.0, cash: 10000.0, positions_value: 0.0 },
            EquityPoint { timestamp: 86400000, equity: 10500.0, cash: 10500.0, positions_value: 0.0 },
            EquityPoint { timestamp: 172800000, equity: 10300.0, cash: 10300.0, positions_value: 0.0 },
            EquityPoint { timestamp: 259200000, equity: 11000.0, cash: 11000.0, positions_value: 0.0 },
        ];

        let trades = vec![
            Trade {
                symbol: "BTCUSDT".into(),
                side: "long".into(),
                quantity: 1.0,
                entry_price: 100.0,
                exit_price: 105.0,
                entry_time: 0,
                exit_time: 86400000,
                pnl: 500.0,
                return_pct: 5.0,
                commission: 1.0,
                slippage: 0.5,
            },
            Trade {
                symbol: "BTCUSDT".into(),
                side: "long".into(),
                quantity: 1.0,
                entry_price: 105.0,
                exit_price: 103.0,
                entry_time: 86400000,
                exit_time: 172800000,
                pnl: -200.0,
                return_pct: -1.9,
                commission: 1.0,
                slippage: 0.5,
            },
        ];

        let metrics = Metrics::calculate(&equity_curve, &trades, 10000.0, 0.02);

        assert_eq!(metrics.total_trades, 2);
        assert_eq!(metrics.winning_trades, 1);
        assert_eq!(metrics.losing_trades, 1);
        assert!(metrics.total_return > 0.0);
    }
}
