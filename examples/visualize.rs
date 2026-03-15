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

    let mut templates = Vec::new();

    // Founding: walls and gates
    for i in 0..6 {
        templates.push(BuildingTemplate {
            name: format!("wall_tower_{i}"),
            category: Category::Military,
            radius: 2,
            priority: 1.0 - i as f32 * 0.01,
            connections: vec![],
        });
    }

    // Infrastructure
    for i in 0..8 {
        templates.push(BuildingTemplate {
            name: format!("well_{i}"),
            category: Category::Infrastructure,
            radius: 1,
            priority: 0.9 - i as f32 * 0.02,
            connections: vec![],
        });
    }
    for i in 0..4 {
        templates.push(BuildingTemplate {
            name: format!("granary_{i}"),
            category: Category::Infrastructure,
            radius: 2,
            priority: 0.85 - i as f32 * 0.02,
            connections: vec![],
        });
    }

    // Sacred
    templates.push(BuildingTemplate {
        name: "great_temple".into(),
        category: Category::Sacred,
        radius: 3,
        priority: 0.8,
        connections: vec![],
    });
    for i in 0..3 {
        templates.push(BuildingTemplate {
            name: format!("shrine_{i}"),
            category: Category::Sacred,
            radius: 2,
            priority: 0.7 - i as f32 * 0.05,
            connections: vec![],
        });
    }

    // Commercial
    for i in 0..6 {
        templates.push(BuildingTemplate {
            name: format!("market_{i}"),
            category: Category::Commercial,
            radius: 2,
            priority: 0.75 - i as f32 * 0.03,
            connections: vec![],
        });
    }
    for i in 0..8 {
        templates.push(BuildingTemplate {
            name: format!("workshop_{i}"),
            category: Category::Commercial,
            radius: 1,
            priority: 0.5 - i as f32 * 0.02,
            connections: vec![],
        });
    }

    // Residential
    for i in 0..50 {
        templates.push(BuildingTemplate {
            name: format!("house_{i}"),
            category: Category::Residential,
            radius: 1,
            priority: 0.3 - i as f32 * 0.005,
            connections: vec![],
        });
    }
    for i in 0..10 {
        templates.push(BuildingTemplate {
            name: format!("villa_{i}"),
            category: Category::Residential,
            radius: 2,
            priority: 0.35 - i as f32 * 0.01,
            connections: vec![],
        });
    }

    let catalog = AgentCatalog { templates };

    let spec = CitySpec {
        width: 200,
        height: 200,
        city_type: CityType::TradeHub,
        era: Era::Growth,
        beta: 2.5,
        seed: 7,
        erosion: severity.map(|s| ErosionSpec {
            severity: s,
            seed: 7,
        }),
    };

    eprintln!(
        "Generating {} buildings on {}x{} grid...",
        catalog.templates.len(),
        spec.width,
        spec.height
    );
    let city = generate(&spec, &catalog);

    // Build a render grid: each cell is (char, ansi_color_code).
    let (w, h) = (spec.width as usize, spec.height as usize);
    let mut grid: Vec<(char, &str)> = vec![(' ', "0"); w * h];
    let idx = |x: usize, y: usize| y * w + x;

    // 1. Draw roads as thin paths.
    for road in &city.roads {
        for &(rx, ry) in &road.path {
            let (rx, ry) = (rx as usize, ry as usize);
            if rx < w && ry < h {
                grid[idx(rx, ry)] = ('·', "37"); // white dots
            }
        }
    }

    // 2. Draw building outlines (border only) and center marker.
    for b in &city.buildings {
        let r = b.radius as i32;
        let (color, fill, border, center) = match b.category {
            Category::Military => ("31", '░', '▓', '▓'),
            Category::Infrastructure => ("36", '░', '▒', '▒'),
            Category::Sacred => ("33", '░', '█', '█'),
            Category::Commercial => ("32", '░', '▒', '▒'),
            Category::Residential => ("34", ' ', '▪', '▪'),
        };

        for dy in -r..=r {
            for dx in -r..=r {
                let bx = b.x as i32 + dx;
                let by = b.y as i32 + dy;
                if bx >= 0 && by >= 0 && (bx as usize) < w && (by as usize) < h {
                    let on_edge = dx == -r || dx == r || dy == -r || dy == r;
                    let at_center = dx == 0 && dy == 0;
                    let ch = if at_center {
                        center
                    } else if on_edge {
                        border
                    } else {
                        fill
                    };
                    grid[idx(bx as usize, by as usize)] = (ch, color);
                }
            }
        }
    }

    // Crop to bounding box of content + 2 cell margin.
    let mut min_x = w;
    let mut max_x = 0usize;
    let mut min_y = h;
    let mut max_y = 0usize;
    for y in 0..h {
        for x in 0..w {
            if grid[idx(x, y)].0 != ' ' {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }
    let margin = 2;
    min_x = min_x.saturating_sub(margin);
    min_y = min_y.saturating_sub(margin);
    max_x = (max_x + margin).min(w - 1);
    max_y = (max_y + margin).min(h - 1);

    println!();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let (ch, color) = grid[idx(x, y)];
            if ch == ' ' {
                print!(" ");
            } else {
                print!("\x1b[{color}m{ch}\x1b[0m");
            }
        }
        println!();
    }

    println!();
    println!(
        "  \x1b[31m▓\x1b[0m Military  \x1b[36m▒\x1b[0m Infrastructure  \x1b[33m█\x1b[0m Sacred  \x1b[32m▒\x1b[0m Commercial  \x1b[34m▪\x1b[0m Residential  \x1b[37m·\x1b[0m Road"
    );
    println!(
        "  {} buildings, {} roads, score: {:.3}{}",
        city.buildings.len(),
        city.roads.len(),
        city.score.composite,
        severity.map_or(String::new(), |s| format!(", erosion: {:.0}%", s * 100.0)),
    );
    println!();
}
