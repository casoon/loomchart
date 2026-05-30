//! Renko candle transformation.
//!
//! Converts OHLCV candles into Renko bricks. Each brick has a fixed `brick_size`
//! in price units. Up-bricks are bullish; down-bricks are bearish.

use super::types::Candle;

/// Transform raw OHLCV candles into Renko bricks.
///
/// Returns a `Vec<Candle>` where each element represents one brick:
/// - `o` / `c` reflect the brick bottom/top (up) or top/bottom (down)
/// - `h` = top of brick, `l` = bottom of brick
/// - `v` is the sum of volumes of all source candles that contributed to the brick
/// - `time` is the timestamp of the last source candle that completed the brick
pub fn compute_renko(candles: &[Candle], brick_size: f64) -> Vec<Candle> {
    if candles.is_empty() || brick_size <= 0.0 {
        return Vec::new();
    }

    let mut bricks: Vec<Candle> = Vec::new();

    // Anchor to the first candle's close, rounded to the nearest brick boundary
    let first_close = candles[0].c;
    let anchor = (first_close / brick_size).floor() * brick_size;
    let mut last_top = anchor + brick_size;
    let mut last_bot = anchor;
    let mut accumulated_vol = candles[0].v;

    for candle in candles.iter().skip(1) {
        accumulated_vol += candle.v;
        let close = candle.c;

        // Build upward bricks
        while close >= last_top + brick_size {
            let brick_bot = last_top;
            let brick_top = last_top + brick_size;
            bricks.push(Candle {
                time: candle.time,
                o: brick_bot,
                h: brick_top,
                l: brick_bot,
                c: brick_top,
                v: accumulated_vol,
            });
            last_bot = brick_bot;
            last_top = brick_top;
            accumulated_vol = 0.0;
        }
        // Also push a single brick when close crosses into the next level by at least brick_size
        if close >= last_top {
            let brick_bot = last_top;
            let brick_top = last_top + brick_size;
            bricks.push(Candle {
                time: candle.time,
                o: brick_bot,
                h: brick_top,
                l: brick_bot,
                c: brick_top,
                v: accumulated_vol,
            });
            last_bot = brick_bot;
            last_top = brick_top;
            accumulated_vol = 0.0;
            continue;
        }

        // Build downward bricks
        while close <= last_bot - brick_size {
            let brick_top = last_bot;
            let brick_bot = last_bot - brick_size;
            bricks.push(Candle {
                time: candle.time,
                o: brick_top,
                h: brick_top,
                l: brick_bot,
                c: brick_bot,
                v: accumulated_vol,
            });
            last_top = brick_top;
            last_bot = brick_bot;
            accumulated_vol = 0.0;
        }
        if close <= last_bot {
            let brick_top = last_bot;
            let brick_bot = last_bot - brick_size;
            bricks.push(Candle {
                time: candle.time,
                o: brick_top,
                h: brick_top,
                l: brick_bot,
                c: brick_bot,
                v: accumulated_vol,
            });
            last_top = brick_top;
            last_bot = brick_bot;
            accumulated_vol = 0.0;
        }
    }

    bricks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(time: i64, c: f64) -> Candle {
        Candle::new(time, c, c + 1.0, c - 1.0, c, 100.0)
    }

    #[test]
    fn test_up_bricks() {
        let candles: Vec<Candle> = (0..=20)
            .map(|i| make_candle(i, 100.0 + i as f64))
            .collect();
        let bricks = compute_renko(&candles, 5.0);
        // Should produce upward bricks
        assert!(!bricks.is_empty());
        for b in &bricks {
            assert!(b.c > b.o, "Expected up-brick: o={}, c={}", b.o, b.c);
        }
    }

    #[test]
    fn test_down_bricks() {
        let candles: Vec<Candle> = (0..=20)
            .map(|i| make_candle(i, 120.0 - i as f64))
            .collect();
        let bricks = compute_renko(&candles, 5.0);
        assert!(!bricks.is_empty());
        for b in &bricks {
            assert!(b.c < b.o, "Expected down-brick: o={}, c={}", b.o, b.c);
        }
    }

    #[test]
    fn test_direction_change() {
        let mut candles = vec![];
        for i in 0..15 {
            candles.push(make_candle(i, 100.0 + i as f64));
        }
        for i in 0..15 {
            candles.push(make_candle(15 + i, 115.0 - i as f64));
        }
        let bricks = compute_renko(&candles, 5.0);
        assert!(!bricks.is_empty());
        let up_count = bricks.iter().filter(|b| b.c > b.o).count();
        let down_count = bricks.iter().filter(|b| b.c < b.o).count();
        assert!(up_count > 0);
        assert!(down_count > 0);
    }
}
