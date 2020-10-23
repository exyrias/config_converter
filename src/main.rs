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
    ConvertError,
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
                    match x.as_f64().unwrap() {
                        f64::NAN => serde_json::Value::String(String::from(".nan")),
                        f64::INFINITY => serde_json::Value::String(String::from(".inf")),
                        f64::NEG_INFINITY => serde_json::Value::String(String::from("-.inf")),
                        x => serde_json::Value::Number(serde_json::Number::from_f64(x).unwrap()),
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

fn read_stream<R>(r: &mut R) -> Result<String, Box<dyn std::error::Error>>
where
    R: std::io::Read,
{
    let mut contents = String::new();
    let _size = r.read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse_args() -> Result<Option<String>, Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        Ok(None)
    } else if args.len() == 2 {
        Ok(Some(args[1].clone()))
    } else {
        Err(Error::ArgumentsError)
    }
}

fn load_data() -> Result<String, String> {
    match parse_args() {
        Ok(None) => match read_stream(&mut std::io::stdin()) {
            Ok(s) => Ok(s),
            Err(_) => Err(String::from("Cannot read standard input.")),
        },
        Ok(Some(fname)) => match File::open(&fname) {
            Ok(mut f) => match read_stream(&mut f) {
                Ok(s) => Ok(s),
                Err(_) => Err(format!("Cannot read file \"{}\".", fname)),
            },
            Err(_) => {
                return Err(format!("Cannot open file \"{}\".", fname));
            }
        },
        Err(Error::ArgumentsError) => {
            return Err(String::from("Arguments is incorrect."));
        },
        _ => panic!("Never occurs")
    }
}

fn convert<S, T> (parse: impl FnOnce(&String) -> Result<S,Error>, to_string: impl FnOnce(&T)->Result<String, Error>) 
where
    S: Converter<T>,
    T: serde::ser::Serialize
    {
    std::process::exit(match load_data() {
        Ok(contents) => match parse(&contents) {
            Ok(value) => {
                let converted= value.convert();
                match to_string(&converted) {
                    Ok(s) => {
                        println!("{}", s);
                        0
                    }
                    Err(e) => {
                        eprintln!("{:?}", e);
                        1
                    }
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
                1
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    });
}

fn main() {
    // json_to_yaml();
    convert(load_yaml, |x| {
        match serde_json::to_string(x) {
            Ok(x) => Ok(x),
            Err(_) => Err(Error::ConvertError),
        }
    });
}