mod converter {
    extern crate serde;
    extern crate serde_derive;
    extern crate serde_json;
    extern crate serde_yaml;
    extern crate toml;

    use std::iter::FromIterator;

    trait Converter<T> {
        fn convert(&self) -> T;
    }

    pub fn load_json(s: &String) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        match serde_json::from_str(s) {
            Ok(dat) => Ok(dat),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub fn load_yaml(s: &String) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
        match serde_yaml::from_str(s) {
            Ok(dat) => Ok(dat),
            Err(e) => Err(Box::new(e)),
        }
    }

    impl Converter<serde_yaml::Value> for serde_json::Value {
        fn convert(&self) -> serde_yaml::Value {
            match self {
                serde_json::Value::Array(x) => {
                    serde_yaml::Value::Sequence(x.iter().map(|v| v.convert()).collect())
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
                        .map(|(k, v)| (serde_yaml::Value::String(k.clone()), v.convert()));
                    serde_yaml::Value::Mapping(serde_yaml::Mapping::from_iter(iter))
                }
                serde_json::Value::String(x) => serde_yaml::Value::String(x.clone()),
            }
        }
    }

    impl Converter<serde_json::Value> for serde_yaml::Value {
        fn convert(&self) -> serde_json::Value {
            match self {
                serde_yaml::Value::Bool(x) => serde_json::Value::Bool(*x),
                serde_yaml::Value::Mapping(x) => {
                    serde_json::Value::Object(serde_json::Map::from_iter(x.iter().map(|(k, v)| {
                        let k = match k {
                            serde_yaml::Value::String(k) => k.clone(),
                            serde_yaml::Value::Number(x) => x.to_string(),
                            serde_yaml::Value::Bool(k) => k.to_string(),
                            _ => panic!("Cannot convert yaml to json"),
                        };
                        (k, v.convert())
                    })))
                }
                serde_yaml::Value::Null => serde_json::Value::Null,
                serde_yaml::Value::Number(x) => {
                    if x.is_u64() {
                        serde_json::Value::Number(serde_json::Number::from(x.as_u64().unwrap()))
                    } else if x.is_i64() {
                        serde_json::Value::Number(serde_json::Number::from(x.as_i64().unwrap()))
                    } else if x.is_f64() {
                        let x = x.as_f64().unwrap();
                        if x.is_nan() {
                            serde_json::Value::String(String::from(".nan"))
                        } else if x.is_infinite() {
                            if x > 0.0 {
                                serde_json::Value::String(String::from(".inf"))
                            } else {
                                serde_json::Value::String(String::from("-.inf"))
                            }
                        } else {
                            serde_json::Value::Number(serde_json::Number::from_f64(x).unwrap())
                        }
                    } else {
                        panic!("Cannot convert yaml to json");
                    }
                }
                serde_yaml::Value::Sequence(x) => {
                    serde_json::Value::Array(x.iter().map(|x| x.convert()).collect())
                }
                serde_yaml::Value::String(x) => serde_json::Value::String(x.clone()),
            }
        }
    }

    #[test]
    fn yaml2json() {
        let x = serde_yaml::Value::Number(serde_yaml::Number::from(1));
        assert_eq!(
            x.convert(),
            serde_json::Value::Number(serde_json::Number::from(1))
        );

        let x = serde_yaml::Value::Number(serde_yaml::Number::from(f64::NAN));
        assert_eq!(x.convert(), serde_json::Value::String(String::from(".nan")));

        let x = serde_yaml::Value::Number(serde_yaml::Number::from(f64::INFINITY));
        assert_eq!(x.convert(), serde_json::Value::String(String::from(".inf")));

        let x = serde_yaml::Value::Number(serde_yaml::Number::from(f64::NEG_INFINITY));
        assert_eq!(
            x.convert(),
            serde_json::Value::String(String::from("-.inf"))
        );
    }

    pub enum ConvertType {
        Yaml2Json,
        Yaml2JsonPretty,
        Json2Yaml,
        Json2JsonPretty,
    }

    pub fn convert_string(
        cnvt_type: ConvertType,
        data: &String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let s = match cnvt_type {
            ConvertType::Yaml2Json => serde_json::to_string(&load_yaml(&data)?.convert())?,
            ConvertType::Yaml2JsonPretty => {
                serde_json::to_string_pretty(&load_yaml(&data)?.convert())?
            }
            ConvertType::Json2Yaml => serde_yaml::to_string(&load_json(&data)?.convert())?,
            ConvertType::Json2JsonPretty => {
                serde_json::to_string_pretty(&load_json(&data)?.convert())?
            }
        };
        Ok(s)
    }
}

mod program {
    mod read_utils {
        use std::error::Error;
        use std::fs::File;

        pub fn input_selector(
            file_name: Option<String>,
        ) -> Result<Box<dyn std::io::Read>, Box<dyn Error>> {
            match file_name {
                None => Ok(Box::new(std::io::stdin())),
                Some(x) => Ok(Box::new(File::open(&x)?)),
            }
        }

        pub fn read_stream<R>(r: &mut R) -> Result<String, Box<dyn Error>>
        where
            R: std::io::Read,
        {
            let mut contents = String::new();
            let _size = r.read_to_string(&mut contents)?;
            Ok(contents)
        }
    }

    use crate::converter::*;
    use std::env;
    use std::error::Error;

    #[derive(Debug)]
    enum RunError {
        ArgumentsError,
        ConvertTypeParseError,
    }
    impl std::fmt::Display for RunError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
            match *self {
                Self::ArgumentsError => f.write_str("Invalid arguments"),
                Self::ConvertTypeParseError => f.write_str("Invalid convert type"),
            }
        }
    }
    impl Error for RunError {}

    fn parse_args() -> Result<(String, Option<String>), Box<dyn Error>> {
        let args: Vec<String> = env::args().collect();
        if args.len() == 2 {
            Ok((args[1].clone(), None))
        } else if args.len() == 3 {
            Ok((args[1].clone(), Some(args[2].clone())))
        } else {
            Err(Box::new(RunError::ArgumentsError))
        }
    }

    fn convert_type(t: &str) -> Result<ConvertType, Box<dyn std::error::Error>> {
        match t {
            "y2j" => Ok(ConvertType::Yaml2Json),
            "y2jp" => Ok(ConvertType::Yaml2JsonPretty),
            "j2y" => Ok(ConvertType::Json2Yaml),
            "j2jp" => Ok(ConvertType::Json2JsonPretty),
            _ => Err(Box::new(RunError::ConvertTypeParseError)),
        }
    }

    fn convert() -> Result<String, Box<dyn Error>> {
        let (cnvt_type, input_stream) = parse_args()?;
        let cnvt_type = convert_type(&cnvt_type)?;
        let mut input_stream = read_utils::input_selector(input_stream)?;
        let data = read_utils::read_stream(&mut input_stream)?;
        let s = convert_string(cnvt_type, &data)?;
        Ok(s)
    }

    fn show_converted(s: String) {
        println!("{}", s);
    }

    fn show_error(e: Box<dyn Error>) {
        eprintln!("{}", e);
    }

    pub fn run() {
        match convert() {
            Ok(s) => show_converted(s),
            Err(e) => show_error(e),
        }
    }
}

fn main() {
    program::run();
}
