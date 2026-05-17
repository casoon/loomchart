use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use chartcore::core::{Candle, CandleGenerator, ChartRenderer, GeneratorConfig, InvalidationLevel};

fn make_candles(n: usize) -> Vec<Candle> {
    let config = GeneratorConfig::crypto().with_seed(42);
    let mut gen = CandleGenerator::new(config);
    gen.generate(n)
}

fn bench_initial_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("initial_load");
    for size in [1_000usize, 10_000, 50_000, 100_000] {
        let candles = make_candles(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &candles, |b, c| {
            b.iter(|| {
                let mut renderer = ChartRenderer::new(1280.0, 720.0);
                renderer.invalidate(InvalidationLevel::Full);
                black_box(renderer.render(c))
            });
        });
    }
    group.finish();
}

fn bench_crosshair_update(c: &mut Criterion) {
    let candles = make_candles(1_000);
    let mut renderer = ChartRenderer::new(1280.0, 720.0);
    renderer.invalidate(InvalidationLevel::Full);
    renderer.render(&candles); // initial render

    c.bench_function("crosshair_update", |b| {
        b.iter(|| {
            renderer.mark_crosshair_dirty();
            black_box(renderer.render(&candles))
        });
    });
}

criterion_group!(benches, bench_initial_load, bench_crosshair_update);
criterion_main!(benches);
