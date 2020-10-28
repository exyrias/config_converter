extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
extern crate toml;

use std::env;
use std::fs::File;
use std::iter::FromIterator;

#[derive(Debug)]
enum Error {
    ArgumentsError,
    InvalidData,
    ReadInputError,
    ConvertError,
    ConvertTypeParseError,
    FileOpenError,
}

fn load_json(s: &String) -> Result<serde_json::Value, Error> {
    match serde_json::from_str(s) {
        Ok(dat) => Ok(dat),
        Err(_) => Err(Error::InvalidData),
    }
}

trait Converter<T> {
    fn convert(&self) -> T;
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

fn load_yaml(s: &String) -> Result<serde_yaml::Value, Error> {
    match serde_yaml::from_str(s) {
        Ok(dat) => Ok(dat),
        Err(_) => Err(Error::InvalidData),
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

fn read_stream<R>(r: &mut R) -> Result<String, Box<dyn std::error::Error>>
where
    R: std::io::Read,
{
    let mut contents = String::new();
    let _size = r.read_to_string(&mut contents)?;
    Ok(contents)
}

enum ConvertType {
    Yaml2Json,
    Json2Yaml,
}

fn convert_type(t: &str) -> Result<ConvertType, Error> {
    if t == "y2j" {
        Ok(ConvertType::Yaml2Json)
    } else if t == "j2y" {
        Ok(ConvertType::Json2Yaml)
    } else {
        Err(Error::ConvertTypeParseError)
    }
}

fn parse_args() -> Result<(String, Option<String>), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        Ok((args[1].clone(), None))
    } else if args.len() == 3 {
        Ok((args[1].clone(), Some(args[2].clone())))
    } else {
        Err(Error::ArgumentsError)
    }
}

fn input_selector(file_name: Option<String>) -> Result<Box<dyn std::io::Read>, Error> {
    match file_name {
        None => Ok(Box::new(std::io::stdin())),
        Some(x) => Ok(Box::new(File::open(&x).or(Err(Error::FileOpenError))?)),
    }
}

fn run() -> Result<(), Error> {
    let (cnvt_type, input_stream) = parse_args()?;
    let cnvt_type = convert_type(&cnvt_type).or(Err(Error::ArgumentsError))?;
    let mut input_stream = input_selector(input_stream)?;
    let data = read_stream(&mut input_stream).or(Err(Error::ReadInputError))?;

    let s = match cnvt_type {
        ConvertType::Yaml2Json => serde_json::to_string_pretty(&load_yaml(&data)?.convert())
            .or(Err(Error::ConvertError))?,
        ConvertType::Json2Yaml => {
            serde_yaml::to_string(&load_json(&data)?.convert()).or(Err(Error::ConvertError))?
        }
    };
    println!("{}", s);

    Ok(())
}

fn show_error(e: Error) {
    match e {
        Error::ArgumentsError => {
            eprintln!("Invalid arguments.");
        }
        Error::ConvertError => {
            eprintln!("Cannot convert format.");
        }
        Error::ConvertTypeParseError => {
            eprintln!("Invalid convert type.");
        }
        Error::FileOpenError => {
            eprintln!("Cannot open file.");
        }
        Error::InvalidData => {
            eprintln!("Invalid data format.");
        }
        Error::ReadInputError => {
            eprintln!("Cannot read data.");
        }
    }
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => show_error(e),
    }
}
