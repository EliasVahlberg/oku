//! Visualize a generated city as a colored Unicode grid in the terminal.
//!
//! Run: `cargo run --example visualize`
//! With erosion: `cargo run --example visualize -- --erode 0.4`

use oku::*;

fn main() {
    let severity = std::env::args()
        .position(|a| a == "--erode")
        .and_then(|i| std::env::args().nth(i + 1))
        .and_then(|s| s.parse::<f32>().ok());

    let catalog = AgentCatalog {
        templates: vec![
            // Founding
            BuildingTemplate {
                name: "wall_tower".into(),
                category: Category::Military,
                radius: 2,
                priority: 1.0,
                connections: vec![],
            },
            BuildingTemplate {
                name: "gate".into(),
                category: Category::Military,
                radius: 1,
                priority: 0.95,
                connections: vec![],
            },
            BuildingTemplate {
                name: "well".into(),
                category: Category::Infrastructure,
                radius: 1,
                priority: 0.9,
                connections: vec![],
            },
            // Core
            BuildingTemplate {
                name: "temple".into(),
                category: Category::Sacred,
                radius: 3,
                priority: 0.7,
                connections: vec![],
            },
            BuildingTemplate {
                name: "market".into(),
                category: Category::Commercial,
                radius: 2,
                priority: 0.75,
                connections: vec![],
            },
            // Growth
            BuildingTemplate {
                name: "house".into(),
                category: Category::Residential,
                radius: 1,
                priority: 0.3,
                connections: vec![],
            },
            BuildingTemplate {
                name: "house".into(),
                category: Category::Residential,
                radius: 1,
                priority: 0.25,
                connections: vec![],
            },
            BuildingTemplate {
                name: "house".into(),
                category: Category::Residential,
                radius: 1,
                priority: 0.2,
                connections: vec![],
            },
            BuildingTemplate {
                name: "house".into(),
                category: Category::Residential,
                radius: 1,
                priority: 0.15,
                connections: vec![],
            },
            BuildingTemplate {
                name: "workshop".into(),
                category: Category::Commercial,
                radius: 1,
                priority: 0.4,
                connections: vec![],
            },
            BuildingTemplate {
                name: "granary".into(),
                category: Category::Infrastructure,
                radius: 2,
                priority: 0.6,
                connections: vec![],
            },
        ],
    };

    let spec = CitySpec {
        width: 30,
        height: 25,
        city_type: CityType::TradeHub,
        era: Era::Growth,
        beta: 2.5,
        seed: 7,
        erosion: severity.map(|s| ErosionSpec { severity: s, seed: 7 }),
    };

    let city = generate(&spec, &catalog);
    let grid = city.to_semantic_grid();

    // Legend colors (ANSI 256-color)
    // Military=red, Infrastructure=cyan, Sacred=yellow, Commercial=green, Residential=blue, Road=gray
    println!();
    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = &grid.cells[(y * grid.width + x) as usize];
            match cell.tile {
                Tile::Building => {
                    let bi = cell.building_index.unwrap();
                    let cat = city.buildings[bi].category;
                    let (color, ch) = match cat {
                        Category::Military => ("31", '▓'),       // red
                        Category::Infrastructure => ("36", '▒'), // cyan
                        Category::Sacred => ("33", '█'),         // yellow
                        Category::Commercial => ("32", '▒'),     // green
                        Category::Residential => ("34", '░'),    // blue
                    };
                    print!("\x1b[{color}m{ch}\x1b[0m");
                }
                Tile::Road => print!("\x1b[90m·\x1b[0m"),
                Tile::Empty => print!(" "),
            }
        }
        println!();
    }

    // Legend
    println!();
    println!(
        "  \x1b[31m▓\x1b[0m Military  \x1b[36m▒\x1b[0m Infrastructure  \x1b[33m█\x1b[0m Sacred  \x1b[32m▒\x1b[0m Commercial  \x1b[34m░\x1b[0m Residential  \x1b[90m·\x1b[0m Road"
    );
    println!(
        "  {} buildings, {} roads, score: {:.3}{}",
        city.buildings.len(),
        city.roads.len(),
        city.score,
        severity.map_or(String::new(), |s| format!(", erosion: {:.0}%", s * 100.0)),
    );
    println!();
}
