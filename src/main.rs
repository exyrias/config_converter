extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
extern crate toml;

use std::iter::FromIterator;

fn load_json() -> serde_json::Result<serde_json::Value> {
    let s = r#"
    [{"abc": 1.0, "1":0.1, "bff": "de"},
    {"bbb":"bc"}]"#;
    let dat: serde_json::Value = serde_json::from_str(s)?;
    Ok(dat)
}

trait ToYaml {
    fn to_yaml(&self) -> serde_yaml::Value;
}
impl ToYaml for serde_json::Value {
    fn to_yaml(&self) -> serde_yaml::Value {
        match self {
            serde_json::Value::Array(x) => {
                serde_yaml::Value::Sequence(x.iter().map(|v| v.to_yaml()).collect())
            }
            serde_json::Value::Bool(x) => serde_yaml::Value::Bool(*x),
            serde_json::Value::Null => serde_yaml::Value::Null,
            serde_json::Value::Number(x) => {
                let num = if x.is_u64() {
                    serde_yaml::Number::from(x.as_u64().unwrap())
                } else if x.is_i64() {
                    serde_yaml::Number::from(x.as_i64().unwrap())
                } else if x.is_f64() {
                    serde_yaml::Number::from(x.as_f64().unwrap())
                } else {
                    panic!("Error: cannot covert json to yaml");
                };
                serde_yaml::Value::Number(num)
            }
            serde_json::Value::Object(x) => {
                let iter = x
                    .into_iter()
                    .map(|(k, v)| (serde_yaml::Value::String(k.clone()), v.to_yaml()));
                serde_yaml::Value::Mapping(serde_yaml::Mapping::from_iter(iter))
            }
            serde_json::Value::String(x) => serde_yaml::Value::String(x.clone()),
        }
    }
}

fn main() {
    let json = load_json().unwrap();
    println!("{}", json.to_string());
    let yaml = json.to_yaml();
    println!("{}", serde_yaml::to_string(&yaml).unwrap());
}
