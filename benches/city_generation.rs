use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oku::*;
use std::time::Duration;

fn catalog(n: usize) -> AgentCatalog {
    let mut templates = Vec::new();
    let mut push = |prefix: &str, cat: Category, radius: u32, count: usize, base_pri: f32| {
        for i in 0..count {
            templates.push(BuildingTemplate {
                name: format!("{prefix}_{i}"),
                category: cat,
                radius,
                priority: base_pri - i as f32 * 0.005,
                connections: vec![],
                material: Material::Stone,
            });
        }
    };

    push("wall", Category::Military, 2, (n / 16).max(1), 1.0);
    push("well", Category::Infrastructure, 1, (n / 8).max(1), 0.9);
    push("temple", Category::Sacred, 3, (n / 24).max(1), 0.8);
    push("market", Category::Commercial, 2, (n / 12).max(1), 0.75);
    push("workshop", Category::Commercial, 1, (n / 12).max(1), 0.5);
    push("house", Category::Residential, 1, n / 2, 0.3);
    push("villa", Category::Residential, 2, n / 10, 0.35);

    AgentCatalog { templates }
}

fn spec(width: u32, height: u32) -> CitySpec {
    CitySpec {
        width,
        height,
        city_type: CityType::TradeHub,
        era: Era::Growth,
        beta: 2.5,
        seed: 42,
        erosion: None,
    }
}

fn bench_generate(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));

    // Keep scales small — ogun SA is expensive per iteration
    let scales: &[(&str, usize, u32)] =
        &[("10x30", 10, 30), ("30x80", 30, 80), ("60x120", 60, 120)];

    for &(label, n, size) in scales {
        let cat = catalog(n);
        let s = spec(size, size);
        group.bench_with_input(BenchmarkId::new("full", label), &(s, cat), |b, (s, cat)| {
            b.iter(|| generate(s, cat));
        });
    }
    group.finish();
}

fn bench_erosion(c: &mut Criterion) {
    let mut group = c.benchmark_group("erosion");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));

    let cat = catalog(30);
    for severity in [0.3, 0.6, 0.9] {
        let s = CitySpec {
            erosion: Some(ErosionSpec { severity, seed: 42 }),
            ..spec(80, 80)
        };
        group.bench_with_input(
            BenchmarkId::new("severity", format!("{:.0}pct", severity * 100.0)),
            &(s, &cat),
            |b, (s, cat)| {
                b.iter(|| generate(s, cat));
            },
        );
    }
    group.finish();
}

fn bench_output(c: &mut Criterion) {
    let cat = catalog(30);
    let s = spec(80, 80);
    let city = generate(&s, &cat);

    let mut group = c.benchmark_group("output");
    group.bench_function("to_tilemap", |b| b.iter(|| city.to_tilemap()));
    group.bench_function("to_semantic_grid", |b| b.iter(|| city.to_semantic_grid()));
    group.finish();
}

criterion_group!(benches, bench_generate, bench_erosion, bench_output);
criterion_main!(benches);
