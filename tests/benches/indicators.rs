//! Benchmark tests for indicators.
//!
//! Run with: cargo bench -p loom-tests

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use loom_core::Candle;
use loom_indicators::prelude::*;

/// Generate test candles
fn generate_candles(count: usize) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(count);
    let mut price = 100.0;

    for i in 0..count {
        let change = (i as f64 * 0.1).sin() * 2.0;
        let open = price;
        let close = price + change;
        let high = open.max(close) + 1.0;
        let low = open.min(close) - 1.0;

        candles.push(Candle::new(
            (i as i64) * 60000,
            open,
            high,
            low,
            close,
            1000.0,
        ));
        price = close;
    }

    candles
}

fn bench_sma(c: &mut Criterion) {
    let candles = generate_candles(10_000);

    let mut group = c.benchmark_group("SMA");
    for period in [14, 50, 200].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(period),
            period,
            |b, &period| {
                b.iter(|| {
                    let mut sma = Sma::new(period);
                    for candle in &candles {
                        black_box(sma.next(candle.close));
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_ema(c: &mut Criterion) {
    let candles = generate_candles(10_000);

    let mut group = c.benchmark_group("EMA");
    for period in [14, 50, 200].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(period),
            period,
            |b, &period| {
                b.iter(|| {
                    let mut ema = Ema::new(period);
                    for candle in &candles {
                        black_box(ema.next(candle.close));
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_rsi(c: &mut Criterion) {
    let candles = generate_candles(10_000);

    c.bench_function("RSI 14", |b| {
        b.iter(|| {
            let mut rsi = Rsi::new(14);
            for candle in &candles {
                black_box(rsi.next(candle.close));
            }
        });
    });
}

fn bench_macd(c: &mut Criterion) {
    let candles = generate_candles(10_000);

    c.bench_function("MACD 12,26,9", |b| {
        b.iter(|| {
            let mut macd = Macd::new(12, 26, 9);
            for candle in &candles {
                black_box(macd.next(candle.close));
            }
        });
    });
}

fn bench_bollinger_bands(c: &mut Criterion) {
    let candles = generate_candles(10_000);

    c.bench_function("Bollinger Bands 20,2", |b| {
        b.iter(|| {
            let mut bb = BollingerBands::new(20, 2.0);
            for candle in &candles {
                black_box(bb.next(candle.close));
            }
        });
    });
}

fn bench_atr(c: &mut Criterion) {
    let candles = generate_candles(10_000);

    c.bench_function("ATR 14", |b| {
        b.iter(|| {
            let mut atr = Atr::new(14);
            for candle in &candles {
                black_box(atr.next(&candle));
            }
        });
    });
}

fn bench_stochastic(c: &mut Criterion) {
    let candles = generate_candles(10_000);

    c.bench_function("Stochastic 14,3,3", |b| {
        b.iter(|| {
            let mut stoch = Stochastic::new(14, 3, 3);
            for candle in &candles {
                black_box(stoch.next(&candle));
            }
        });
    });
}

fn bench_adx(c: &mut Criterion) {
    let candles = generate_candles(10_000);

    c.bench_function("ADX 14", |b| {
        b.iter(|| {
            let mut adx = Adx::new(14);
            for candle in &candles {
                black_box(adx.next(&candle));
            }
        });
    });
}

fn bench_combined(c: &mut Criterion) {
    let candles = generate_candles(10_000);

    c.bench_function("Combined (EMA+RSI+MACD+BB)", |b| {
        b.iter(|| {
            let mut ema = Ema::new(14);
            let mut rsi = Rsi::new(14);
            let mut macd = Macd::new(12, 26, 9);
            let mut bb = BollingerBands::new(20, 2.0);

            for candle in &candles {
                black_box(ema.next(candle.close));
                black_box(rsi.next(candle.close));
                black_box(macd.next(candle.close));
                black_box(bb.next(candle.close));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_sma,
    bench_ema,
    bench_rsi,
    bench_macd,
    bench_bollinger_bands,
    bench_atr,
    bench_stochastic,
    bench_adx,
    bench_combined,
);

criterion_main!(benches);
