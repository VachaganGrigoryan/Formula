//! `formula-render tools`: the catalogue as JSON, for `tools/shorts` and the
//! `formula-short` skill (valid ids, names, what each tool's `state` accepts).

use formula::formulas::{TOOLS, ToolEnv};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Entry {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub alias: &'static str,
    pub summary: &'static str,
    /// Plain text of each equation line.
    pub equations: Vec<String>,
    /// Fields a scene's `state` may set.
    pub state_params: Vec<String>,
    /// Smallest stage the tool lays out in, in points.
    pub min_size: [f32; 2],
    /// Whether the tool moves on its own (otherwise the video shows a still).
    pub animates: bool,
}

pub fn entries() -> Vec<Entry> {
    let env = ToolEnv::new();
    TOOLS
        .iter()
        .map(|t| {
            let f = (t.make)(&env);
            let mut state_params = f.state_params();
            state_params.sort();
            let min = f.min_size();
            Entry {
                id: t.id,
                name: t.name,
                category: t.category,
                alias: t.alias,
                summary: f.summary(),
                equations: f.equations().iter().map(|e| e.text()).collect(),
                state_params,
                min_size: [min.x, min.y],
                animates: f.is_animating(),
            }
        })
        .collect()
}

pub fn json() -> String {
    serde_json::to_string_pretty(&entries()).expect("catalogue serializes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_is_listed_with_its_state_fields() {
        let all = entries();
        assert_eq!(all.len(), TOOLS.len());
        let lorenz = all.iter().find(|e| e.id == "lorenz").unwrap();
        assert_eq!(lorenz.name, "Lorenz attractor");
        assert!(lorenz.state_params.iter().any(|p| p == "rho"));
        assert!(lorenz.equations.iter().any(|e| e.contains("dx/dt")));
        assert!(lorenz.animates);
        assert!(lorenz.min_size[0] >= 360.0);
        assert!(!lorenz.summary.is_empty());
    }

    #[test]
    fn json_is_an_array_of_ids() {
        let v: serde_json::Value = serde_json::from_str(&json()).unwrap();
        let ids: Vec<&str> = v
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["id"].as_str().unwrap())
            .collect();
        assert!(ids.contains(&"a-star") && ids.contains(&"fourier-epicycles"));
    }
}
