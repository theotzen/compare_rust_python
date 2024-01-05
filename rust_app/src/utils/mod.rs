use serde_yaml;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NestedValue {
    Value(String),
    Map(HashMap<String, NestedValue>),
    List(Vec<NestedValue>),
}

pub type NestedHashMap = HashMap<String, NestedValue>;

pub fn compare_dicts(
    dict_a: &NestedHashMap,
    dict_b: &NestedHashMap,
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let mut left_not_right: Vec<String> = Vec::new();
    let mut right_not_left: Vec<String> = Vec::new();
    let mut same_key_same_value: Vec<String> = Vec::new();
    let mut same_key_diff_value: Vec<String> = Vec::new();

    let mut stack: Vec<(String, &NestedHashMap, &NestedHashMap)> = Vec::new();
    stack.push(("".to_owned(), dict_a, dict_b));

    while let Some((path, current_dict_a, current_dict_b)) = stack.pop() {
        for (key, value_a) in current_dict_a {
            let new_path = if path.is_empty() {
                format!("/{}", key)
            } else {
                format!("{}/{}", path, key)
            };
            match current_dict_b.get(key) {
                None => {
                    info!("{} is in A but not in B", new_path);
                    left_not_right.push(new_path);
                }
                Some(value_b) => match (value_a, value_b) {
                    (NestedValue::Map(sub_dict_a), NestedValue::Map(sub_dict_b)) => {
                        if sub_dict_a.is_empty() && sub_dict_b.is_empty() {
                            info!("Both dictionaries are equally empty");
                            same_key_same_value.push(new_path);
                        } else {
                            info!("Dictionaries aren't empty");
                            stack.push((new_path, sub_dict_a, sub_dict_b));
                        }
                    }
                    (NestedValue::List(list_a), NestedValue::List(list_b)) if list_a == list_b => {
                        info!("{} is in both with the same list", new_path);
                        same_key_same_value.push(new_path);
                    }
                    (NestedValue::List(_), NestedValue::List(_)) => {
                        info!("{} is in both but with different lists", new_path);
                        same_key_diff_value.push(new_path);
                    }
                    (NestedValue::Value(val_a), NestedValue::Value(val_b)) if val_a == val_b => {
                        info!("{} is in both with the same value", new_path);
                        same_key_same_value.push(new_path);
                    }
                    (NestedValue::Value(_val_a), NestedValue::Value(_val_b)) => {
                        info!("{} is in both but with different values", new_path);
                        same_key_diff_value.push(new_path);
                    }
                    _ => {
                        info!("{} is in both but with different types", new_path);
                        same_key_diff_value.push(new_path);
                    }
                },
            }
        }
        for key in current_dict_b.keys() {
            if !current_dict_a.contains_key(key) {
                let new_path = if path.is_empty() {
                    format!("/{}", key)
                } else {
                    format!("{}/{}", path, key)
                };
                info!("{} is in B but not in A", new_path);
                right_not_left.push(new_path);
            }
        }
    }
    (
        left_not_right,
        right_not_left,
        same_key_same_value,
        same_key_diff_value,
    )
}

fn yaml_string_to_nested_hash_map(yaml_content: &str) -> Result<NestedHashMap, serde_yaml::Error> {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml_content)?;
    Ok(convert_yaml_value_to_nested_hash_map(&value))
}

fn convert_yaml_value_to_nested_hash_map(yaml_value: &serde_yaml::Value) -> NestedHashMap {
    match yaml_value {
        serde_yaml::Value::Mapping(mapping) => {
            let mut map = HashMap::new();
            for (key, value) in mapping {
                let key_str = key.as_str().unwrap_or_default().to_string();
                map.insert(key_str, convert_yaml_value_to_nested_value(value));
            }
            map
        }
        _ => HashMap::new(), // If the top-level is not a mapping, return an empty map
    }
}

fn convert_yaml_value_to_nested_value(yaml_value: &serde_yaml::Value) -> NestedValue {
    match yaml_value {
        serde_yaml::Value::Null => NestedValue::Value("null".to_string()),
        serde_yaml::Value::Bool(b) => NestedValue::Value(b.to_string()),
        serde_yaml::Value::Number(num) => NestedValue::Value(num.to_string()),
        serde_yaml::Value::String(s) => NestedValue::Value(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            let list = seq.iter().map(convert_yaml_value_to_nested_value).collect();
            NestedValue::List(list)
        }
        serde_yaml::Value::Mapping(mapping) => {
            let map = mapping
                .iter()
                .map(|(k, v)| {
                    let key_str = k.as_str().unwrap_or_default().to_string();
                    (key_str, convert_yaml_value_to_nested_value(v))
                })
                .collect();
            NestedValue::Map(map)
        }
    }
}

pub fn compare_yaml_strings(
    yaml_a_content: &str,
    yaml_b_content: &str,
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let dict_a = yaml_string_to_nested_hash_map(yaml_a_content).expect("Failed to parse YAML A");
    info!("Loaded yaml_A as dict with {} keys", dict_a.len());

    let dict_b = yaml_string_to_nested_hash_map(yaml_b_content).expect("Failed to parse YAML B");
    info!("Loaded yaml_B as dict with {} keys", dict_b.len());

    let (left_not_right, right_not_left, same_key_same_value, same_key_diff_value) =
        compare_dicts(&dict_a, &dict_b);

    info!(
        "After comparing yamls, leftNotRight is of size : {}",
        left_not_right.len()
    );
    info!(
        "After comparing yamls, rightNotLeft is of size : {}",
        right_not_left.len()
    );
    info!(
        "After comparing yamls, sameKeySameValue is of size : {}",
        same_key_same_value.len()
    );
    info!(
        "After comparing yamls, sameKeyDiffValue is of size : {}",
        same_key_diff_value.len()
    );

    (
        left_not_right,
        right_not_left,
        same_key_same_value,
        same_key_diff_value,
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Once;
    use crate::logger;
    use itertools::Itertools;

    use super::*;

    static INIT: Once = Once::new();

    fn nested_map(map: HashMap<String, NestedValue>) -> NestedValue {
        NestedValue::Map(map)
    }

    fn nested_value(value: &str) -> NestedValue {
        NestedValue::Value(value.to_string())
    }

    pub fn setup_logging_for_tests() {
        INIT.call_once(|| {
            if let Err(e) = logger::setup_logging() {
                eprintln!("Failed to set up logging: {}", e);
                std::process::exit(1);
            }
        });
    }

    #[test]
    fn compare_dicts_basic() {
        setup_logging_for_tests();

        let mut dict_a = NestedHashMap::new();
        dict_a.insert("a".to_string(), nested_value("1"));
        dict_a.insert("b".to_string(), nested_value("2"));
        let mut sub_map_a = NestedHashMap::new();
        sub_map_a.insert("d".to_string(), nested_value("4"));
        sub_map_a.insert("e".to_string(), nested_value("5"));
        dict_a.insert("c".to_string(), nested_map(sub_map_a));

        let mut dict_b = NestedHashMap::new();
        dict_b.insert("a".to_string(), nested_value("1"));
        dict_b.insert("b".to_string(), nested_value("3"));
        dict_b.insert("f".to_string(), nested_value("6"));
        let mut sub_map_b = NestedHashMap::new();
        sub_map_b.insert("d".to_string(), nested_value("4"));
        sub_map_b.insert("g".to_string(), nested_value("7"));
        dict_b.insert("c".to_string(), nested_map(sub_map_b));

        let (left_not_right, right_not_left, same_key_same_value, same_key_diff_value) =
            compare_dicts(&dict_a, &dict_b);

        assert_eq!(left_not_right, vec!["/c/e"]);
        assert_eq!(right_not_left, vec!["/f", "/c/g"]);
        assert_eq!(same_key_same_value, vec!["/a", "/c/d"]);
        assert_eq!(same_key_diff_value, vec!["/b"]);
    }

    #[test]
    fn compare_dicts_diff_keys_in_nested() {
        setup_logging_for_tests();

        let mut dict_c_1 = NestedHashMap::new();
        dict_c_1.insert("a".to_string(), nested_value("1"));
        dict_c_1.insert("b".to_string(), nested_value("2"));
        let mut sub_map_c_1 = NestedHashMap::new();
        sub_map_c_1.insert("d".to_string(), nested_value("4"));
        sub_map_c_1.insert("e".to_string(), nested_value("5"));
        sub_map_c_1.insert("f".to_string(), nested_value("6"));
        sub_map_c_1.insert("h".to_string(), nested_value("10"));
        dict_c_1.insert("c".to_string(), nested_map(sub_map_c_1));

        let mut dict_d_1 = NestedHashMap::new();
        dict_d_1.insert("a".to_string(), nested_value("1"));
        dict_d_1.insert("b".to_string(), nested_value("2"));
        let mut sub_map_d_1 = NestedHashMap::new();
        sub_map_d_1.insert("d".to_string(), nested_value("4"));
        sub_map_d_1.insert("e".to_string(), nested_value("5"));
        sub_map_d_1.insert("f".to_string(), nested_value("7"));
        sub_map_d_1.insert("g".to_string(), nested_value("10"));
        dict_d_1.insert("c".to_string(), nested_map(sub_map_d_1));

        let (left_not_right, right_not_left, same_key_same_value, same_key_diff_value) =
            compare_dicts(&dict_c_1, &dict_d_1);

        assert_eq!(left_not_right, vec!["/c/h"]);
        assert_eq!(right_not_left, vec!["/c/g"]);
        assert_eq!(
            same_key_same_value.iter().sorted().collect::<Vec<_>>(),
            vec!["/a", "/b", "/c/d", "/c/e"]
        );
        assert_eq!(same_key_diff_value, vec!["/c/f"]);
    }

    #[test]
    fn compare_dicts_all_diff() {
        setup_logging_for_tests();

        let mut dict_e = NestedHashMap::new();
        dict_e.insert("a".to_string(), nested_value("1"));
        dict_e.insert("b".to_string(), nested_value("2"));
        let mut sub_map_e = NestedHashMap::new();
        sub_map_e.insert("d".to_string(), nested_value("4"));
        sub_map_e.insert("e".to_string(), nested_value("5"));
        dict_e.insert("c".to_string(), nested_map(sub_map_e));

        let mut dict_f = NestedHashMap::new();
        dict_f.insert("g".to_string(), nested_value("6"));
        dict_f.insert("h".to_string(), nested_value("7"));
        let mut sub_map_f = NestedHashMap::new();
        sub_map_f.insert("j".to_string(), nested_value("8"));
        sub_map_f.insert("k".to_string(), nested_value("9"));
        dict_f.insert("i".to_string(), nested_map(sub_map_f));

        let (left_not_right, right_not_left, same_key_same_value, same_key_diff_value) =
            compare_dicts(&dict_e, &dict_f);

        assert_eq!(
            left_not_right.iter().sorted().collect::<Vec<_>>(),
            vec!["/a", "/b", "/c"]
        );
        assert_eq!(
            right_not_left.iter().sorted().collect::<Vec<_>>(),
            vec!["/g", "/h", "/i"]
        );
        assert_eq!(
            same_key_same_value.iter().sorted().collect::<Vec<_>>(),
            Vec::<&str>::new()
        );
        assert_eq!(
            same_key_diff_value.iter().sorted().collect::<Vec<_>>(),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn test_compare_dicts_with_empty() {
        let mut dict_g = NestedHashMap::new();
        let mut sub_map_g = NestedHashMap::new();
        sub_map_g.insert(
            "live-reloaded-config".to_string(),
            nested_map(NestedHashMap::new()),
        );
        sub_map_g.insert(
            "rolling-restart-config".to_string(),
            nested_map(NestedHashMap::new()),
        );
        dict_g.insert("0.0.0".to_string(), nested_map(sub_map_g));

        let dict_h = dict_g.clone(); // Since the two maps are the same, we can just clone dict_g

        let (left_not_right, right_not_left, same_key_same_value, same_key_diff_value) =
            compare_dicts(&dict_g, &dict_h);

        assert_eq!(left_not_right, Vec::<String>::new());
        assert_eq!(right_not_left, Vec::<String>::new());
        assert_eq!(
            same_key_same_value,
            vec![
                "/0.0.0/live-reloaded-config",
                "/0.0.0/rolling-restart-config"
            ]
        );
        assert_eq!(same_key_diff_value, Vec::<String>::new());
    }

    #[test]
    fn test_compare_dicts_with_diff_list() {
        let mut dict_a = NestedHashMap::new();
        dict_a.insert(
            "versions".to_string(),
            NestedValue::List(vec![
                NestedValue::Value("1.0.0".to_string()),
                NestedValue::Value("1.0.1".to_string()),
            ]),
        );

        let mut dict_b = NestedHashMap::new();
        dict_b.insert(
            "versions".to_string(),
            NestedValue::List(vec![
                NestedValue::Value("1.0.0".to_string()),
                NestedValue::Value("1.0.2".to_string()),
            ]),
        );

        let (left_not_right, right_not_left, same_key_same_value, same_key_diff_value) =
            compare_dicts(&dict_a, &dict_b);

        assert_eq!(left_not_right, Vec::<String>::new());
        assert_eq!(right_not_left, Vec::<String>::new());
        assert_eq!(same_key_same_value, Vec::<String>::new());
        assert_eq!(same_key_diff_value, vec!["/versions"]);
    }

    #[test]
    fn test_compare_dicts_with_same_list() {
        let mut dict_a = NestedHashMap::new();
        dict_a.insert(
            "versions".to_string(),
            NestedValue::List(vec![
                NestedValue::Value("1.0.0".to_string()),
                NestedValue::Value("1.0.1".to_string()),
            ]),
        );

        let mut dict_b = NestedHashMap::new();
        dict_b.insert(
            "versions".to_string(),
            NestedValue::List(vec![
                NestedValue::Value("1.0.0".to_string()),
                NestedValue::Value("1.0.1".to_string()),
            ]),
        );

        let (left_not_right, right_not_left, same_key_same_value, same_key_diff_value) =
            compare_dicts(&dict_a, &dict_b);

        assert_eq!(left_not_right, Vec::<String>::new());
        assert_eq!(right_not_left, Vec::<String>::new());
        assert_eq!(same_key_diff_value, Vec::<String>::new());
        assert_eq!(same_key_same_value, vec!["/versions"]);
    }

    #[test]
    fn test_compare_dicts_with_different_list_sizes() {
        let mut dict_a = NestedHashMap::new();
        dict_a.insert(
            "versions".to_string(),
            NestedValue::List(vec![
                NestedValue::Value("1.0.0".to_string()),
                NestedValue::Value("1.0.1".to_string()),
            ]),
        );

        let mut dict_b = NestedHashMap::new();
        dict_b.insert(
            "versions".to_string(),
            NestedValue::List(vec![NestedValue::Value("1.0.0".to_string())]),
        );

        let (left_not_right, right_not_left, same_key_same_value, same_key_diff_value) =
            compare_dicts(&dict_a, &dict_b);

        assert_eq!(left_not_right, Vec::<String>::new());
        assert_eq!(right_not_left, Vec::<String>::new());
        assert_eq!(same_key_same_value, Vec::<String>::new());
        assert_eq!(same_key_diff_value, vec!["/versions"]);
    }

    #[test]
    fn test_compare_yaml_strings_basic() {
        let yaml_a_content = r#"
        versions:
          - "1.0.0"
          - "1.0.1"
        features:
          - "feature1"
          - "feature2"
        "#;

        let yaml_b_content = r#"
        versions:
          - "1.0.0"
          - "1.0.2"
        features:
          - "feature1"
          - "feature2"
        "#;

        let dict_a = yaml_string_to_nested_hash_map(yaml_a_content).unwrap();
        let dict_b = yaml_string_to_nested_hash_map(yaml_b_content).unwrap();

        let (left_not_right, right_not_left, same_key_same_value, same_key_diff_value) =
            compare_dicts(&dict_a, &dict_b);

        assert_eq!(left_not_right, Vec::<String>::new());
        assert_eq!(right_not_left, Vec::<String>::new());
        assert_eq!(same_key_same_value, vec!["/features"]);
        assert_eq!(same_key_diff_value, vec!["/versions"]);
    }

    #[test]
    fn test_compare_yaml_strings_with_empty() {
        let yaml_a_content = r#"
        config:
          nested_config: {}
        "#;

        let yaml_b_content = r#"
        config:
          nested_config: {}
        "#;

        let dict_a = yaml_string_to_nested_hash_map(yaml_a_content).unwrap();
        let dict_b = yaml_string_to_nested_hash_map(yaml_b_content).unwrap();

        let (left_not_right, right_not_left, same_key_same_value, same_key_diff_value) =
            compare_dicts(&dict_a, &dict_b);

        assert_eq!(left_not_right, Vec::<String>::new());
        assert_eq!(right_not_left, Vec::<String>::new());
        assert_eq!(same_key_same_value, vec!["/config/nested_config"]);
        assert_eq!(same_key_diff_value, Vec::<String>::new());
    }
}
