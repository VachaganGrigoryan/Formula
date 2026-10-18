//! Helpers for saving and restoring a formula's settings
//! ([`crate::formula::Formula::save_state`] / `load_state`).
//!
//! A formula describes what it saves as a serde struct. Loading goes through
//! [`merge`], which takes each field on its own, so one bad value in a sheet
//! keeps the current value of that field and nothing else.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// `current` with every top-level field of `v` applied that deserializes on
/// its own. Unknown fields, fields of the wrong type and a `v` that is not an
/// object leave the current values.
pub fn merge<T: Serialize + DeserializeOwned + Clone>(current: &T, v: &Value) -> T {
    let (Ok(Value::Object(mut base)), Some(fields)) =
        (serde_json::to_value(current), v.as_object())
    else {
        return current.clone();
    };
    let mut result = current.clone();
    for (key, value) in fields {
        let Some(old) = base.get(key).cloned() else {
            continue;
        };
        base.insert(key.clone(), value.clone());
        match serde_json::from_value::<T>(Value::Object(base.clone())) {
            Ok(t) => result = t,
            Err(_) => {
                base.insert(key.clone(), old);
            }
        }
    }
    result
}

/// The `save_state` / `load_state` pair of a formula, inside its
/// `impl Formula`. The formula provides `fn state(&self) -> State` (a serde
/// struct of what it saves) and `fn apply(&mut self, s: State)`, which clamps
/// every value to its slider range before using it.
#[macro_export]
macro_rules! state_hooks {
    () => {
        fn save_state(&self) -> Option<serde_json::Value> {
            $crate::state::to_value(&self.state())
        }

        fn load_state(&mut self, v: &serde_json::Value) {
            let s = $crate::state::merge(&self.state(), v);
            self.apply(s);
        }
    };
}

/// `v` as JSON, with `f32` fields written in their shortest form (`0.78`,
/// not `0.7799999713897705` as `serde_json::to_value` gives).
pub fn to_value<T: Serialize>(v: &T) -> Option<Value> {
    serde_json::from_str(&serde_json::to_string(v).ok()?).ok()
}

/// A grid as one string per row, the character for each cell from `cell`.
pub fn grid_rows(w: usize, h: usize, cell: impl Fn(usize, usize) -> char) -> Vec<String> {
    (0..h)
        .map(|y| (0..w).map(|x| cell(x, y)).collect())
        .collect()
}

/// The cells of `rows`, indexed `[y][x]`, if every row has `w` characters and
/// there are `h` rows.
pub fn parse_rows(rows: &[String], w: usize, h: usize) -> Option<Vec<Vec<char>>> {
    let grid: Vec<Vec<char>> = rows.iter().map(|r| r.chars().collect()).collect();
    (grid.len() == h && grid.iter().all(|r| r.len() == w)).then_some(grid)
}

/// `v` if it is finite, else `fallback`.
pub fn finite(v: f32, fallback: f32) -> f32 {
    if v.is_finite() { v } else { fallback }
}

/// `v` if it is finite, else `fallback`.
pub fn finite64(v: f64, fallback: f64) -> f64 {
    if v.is_finite() { v } else { fallback }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct S {
        a: f32,
        b: bool,
    }

    #[test]
    fn merge_takes_each_field_on_its_own() {
        let s = S { a: 1.0, b: true };
        assert_eq!(merge(&s, &json!({"a": 2, "b": "x"})), S { a: 2.0, b: true });
        assert_eq!(
            merge(&s, &json!({"b": false, "zzz": 1})),
            S { a: 1.0, b: false }
        );
        assert_eq!(merge(&s, &json!(42)), s);
        assert_eq!(merge(&s, &json!({"a": null})), s);
    }

    #[test]
    fn floats_are_written_short() {
        let v = to_value(&S { a: 0.78, b: false }).unwrap();
        assert_eq!(v.to_string(), r#"{"a":0.78,"b":false}"#);
        assert_eq!(merge(&S { a: 0.0, b: false }, &v).a, 0.78);
    }

    #[test]
    fn rows_round_trip_and_reject_wrong_sizes() {
        let rows = grid_rows(3, 2, |x, y| if x == y { '#' } else { '.' });
        assert_eq!(rows, ["#..", ".#."]);
        assert!(parse_rows(&rows, 3, 2).is_some());
        assert!(parse_rows(&rows, 4, 2).is_none());
        assert!(parse_rows(&rows, 3, 3).is_none());
    }
}
