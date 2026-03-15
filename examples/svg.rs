//! Generate an SVG visualization of a city layout.
//!
//! Run: `cargo run --example svg`

use oku::*;
use std::fmt::Write;

const CELL: u32 = 4;

fn main() {
    let mut templates = Vec::new();
    let mut push = |prefix: &str, cat: Category, radius: u32, count: usize, base_pri: f32| {
        for i in 0..count {
            templates.push(BuildingTemplate {
                name: format!("{prefix}_{i}"),
                category: cat,
                radius,
                priority: base_pri - i as f32 * 0.01,
                connections: vec![],
            });
        }
    };

    push("wall_tower", Category::Military, 2, 6, 1.0);
    push("well", Category::Infrastructure, 1, 8, 0.9);
    push("granary", Category::Infrastructure, 2, 4, 0.85);
    push("great_temple", Category::Sacred, 3, 1, 0.8);
    push("shrine", Category::Sacred, 2, 3, 0.7);
    push("market", Category::Commercial, 2, 6, 0.75);
    push("workshop", Category::Commercial, 1, 8, 0.5);
    push("house", Category::Residential, 1, 50, 0.3);
    push("villa", Category::Residential, 2, 10, 0.35);

    let catalog = AgentCatalog { templates };
    let spec = CitySpec {
        width: 200,
        height: 200,
        city_type: CityType::TradeHub,
        era: Era::Growth,
        beta: 2.5,
        seed: 7,
        erosion: None,
    };

    eprintln!("Generating {} buildings...", catalog.templates.len());
    let city = generate(&spec, &catalog);
    eprintln!(
        "{} buildings placed, {} roads",
        city.buildings.len(),
        city.roads.len()
    );

    // Find bounding box of content.
    let (mut min_x, mut max_x) = (spec.width, 0u32);
    let (mut min_y, mut max_y) = (spec.height, 0u32);

    for b in &city.buildings {
        min_x = min_x.min(b.x.saturating_sub(b.radius));
        max_x = max_x.max(b.x + b.radius);
        min_y = min_y.min(b.y.saturating_sub(b.radius));
        max_y = max_y.max(b.y + b.radius);
    }
    for road in &city.roads {
        for &(rx, ry) in &road.path {
            min_x = min_x.min(rx);
            max_x = max_x.max(rx);
            min_y = min_y.min(ry);
            max_y = max_y.max(ry);
        }
    }

    let margin = 3;
    min_x = min_x.saturating_sub(margin);
    min_y = min_y.saturating_sub(margin);
    max_x = (max_x + margin).min(spec.width - 1);
    max_y = (max_y + margin).min(spec.height - 1);

    let w = (max_x - min_x + 1) * CELL;
    let h = (max_y - min_y + 1) * CELL + 28; // extra space for legend

    let mut svg = String::new();
    let _ = writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">"
    );
    let _ = writeln!(svg, "<rect width=\"{w}\" height=\"{h}\" fill=\"#1a1a2e\"/>");

    // Roads.
    for road in &city.roads {
        for &(rx, ry) in &road.path {
            if rx >= min_x && ry >= min_y {
                let px = (rx - min_x) * CELL;
                let py = (ry - min_y) * CELL;
                let _ = writeln!(
                    svg,
                    "<rect x=\"{px}\" y=\"{py}\" width=\"{CELL}\" height=\"{CELL}\" fill=\"#555566\" opacity=\"0.6\"/>"
                );
            }
        }
    }

    // Buildings.
    for b in &city.buildings {
        let (fill, stroke) = match b.category {
            Category::Military => ("#c0392b", "#e74c3c"),
            Category::Infrastructure => ("#2980b9", "#3498db"),
            Category::Sacred => ("#d4a017", "#f1c40f"),
            Category::Commercial => ("#27ae60", "#2ecc71"),
            Category::Residential => ("#7f8c8d", "#95a5a6"),
        };
        let bx = (b.x.saturating_sub(b.radius) - min_x) * CELL;
        let by = (b.y.saturating_sub(b.radius) - min_y) * CELL;
        let side = (b.radius * 2 + 1) * CELL;
        let _ = writeln!(
            svg,
            "<rect x=\"{bx}\" y=\"{by}\" width=\"{side}\" height=\"{side}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"1\" rx=\"1\"/>"
        );
    }

    // Legend.
    let ly = h - 20;
    let items: &[(&str, &str)] = &[
        ("#c0392b", "Military"),
        ("#2980b9", "Infrastructure"),
        ("#d4a017", "Sacred"),
        ("#27ae60", "Commercial"),
        ("#7f8c8d", "Residential"),
        ("#555566", "Road"),
    ];
    let mut lx = 8u32;
    for &(color, label) in items {
        let _ = writeln!(
            svg,
            "<rect x=\"{lx}\" y=\"{ly}\" width=\"10\" height=\"10\" fill=\"{color}\" rx=\"1\"/>"
        );
        let tx = lx + 14;
        let ty = ly + 9;
        let _ = writeln!(
            svg,
            "<text x=\"{tx}\" y=\"{ty}\" fill=\"#cccccc\" font-family=\"monospace\" font-size=\"10\">{label}</text>"
        );
        lx += 14 + label.len() as u32 * 6 + 12;
    }

    let _ = writeln!(svg, "</svg>");
    std::fs::write("docs/city.svg", &svg).unwrap();
    eprintln!("Wrote docs/city.svg ({w}x{h})");
}
