//! Animal silhouettes bundled with the app.
//!
//! Silhouettes adapted from game-icons.net by Lorc, Delapouite and
//! contributors, licensed CC BY 3.0. Outlines were traced and simplified.

use serde::Deserialize;

use super::math::Point;

#[derive(Debug, Deserialize)]
struct RawAnimal {
    /// Name.
    n: String,
    /// Group.
    g: String,
    /// Flattened outline: x0, y0, x1, y1, …
    p: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct Animal {
    pub name: String,
    pub group: String,
    pub outline: Vec<Point>,
}

pub const GROUPS: [&str; 5] = ["All", "Mammals", "Birds", "Sea life", "Bugs and reptiles"];

const DATA: &str = include_str!("../../../assets/animals.json");

/// The bundled animals, parsed once and shared by every instance.
pub fn all() -> &'static [Animal] {
    static ANIMALS: std::sync::OnceLock<Vec<Animal>> = std::sync::OnceLock::new();
    ANIMALS.get_or_init(load)
}

pub fn load() -> Vec<Animal> {
    let raw: Vec<RawAnimal> = serde_json::from_str(DATA).expect("bundled animals.json is valid");
    raw.into_iter()
        .map(|a| Animal {
            name: a.n,
            group: a.g,
            outline: a.p.as_chunks::<2>().0.to_vec(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_data_loads() {
        let animals = load();
        assert_eq!(animals.len(), 64);
        assert!(animals.iter().all(|a| a.outline.len() >= 3));
        assert!(
            animals
                .iter()
                .all(|a| GROUPS[1..].contains(&a.group.as_str()))
        );
        assert!(animals.iter().any(|a| a.name == "Octopus"));
    }
}
