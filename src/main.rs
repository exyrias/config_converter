#[macro_use]
extern crate clap;

mod converter {
    extern crate serde;
    extern crate serde_derive;
    extern crate serde_json;
    extern crate serde_yaml;
    extern crate toml;
    use serde::{Serialize, Serializer};

    enum Value {
        Yaml(serde_yaml::Value),
        Json(serde_json::Value),
        Toml(toml::Value),
    }

    impl Serialize for Value {
        fn serialize<S>(
            &self,
            serializer: S,
        ) -> std::result::Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
        {
            match self {
                Value::Yaml(v) => v.serialize(serializer),
                Value::Json(v) => v.serialize(serializer),
                Value::Toml(v) => v.serialize(serializer),
            }
        }
    }

    fn load_json(s: &String) -> Result<Value, Box<dyn std::error::Error>> {
        match serde_json::from_str(s) {
            Ok(dat) => Ok(Value::Json(dat)),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn load_yaml(s: &String) -> Result<Value, Box<dyn std::error::Error>> {
        match serde_yaml::from_str(s) {
            Ok(dat) => Ok(Value::Yaml(dat)),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn load_toml(s: &String) -> Result<Value, Box<dyn std::error::Error>> {
        match toml::from_str(s) {
            Ok(dat) => Ok(Value::Toml(dat)),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub enum FileType {
        Yaml,
        Json,
        Toml,
    }
    pub struct ConvertType {
        from: FileType,
        to: FileType,
        prettify: bool,
    }
    impl ConvertType {
        pub fn new(from: FileType, to: FileType, prettify: bool) -> Self {
            Self { from, to, prettify }
        }
    }

    pub fn convert_string(
        cnvt_type: ConvertType,
        data: &String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let v = match cnvt_type.from {
            FileType::Yaml => load_yaml(&data)?,
            FileType::Json => load_json(&data)?,
            FileType::Toml => load_toml(&data)?,
        };
        let s = match (cnvt_type.to, cnvt_type.prettify) {
            (FileType::Yaml, _) => serde_yaml::to_string(&v)?,
            (FileType::Json, false) => serde_json::to_string(&v)?,
            (FileType::Json, true) => serde_json::to_string_pretty(&v)?,
            (FileType::Toml, false) => toml::to_string(&v)?,
            (FileType::Toml, true) => toml::to_string_pretty(&v)?,
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
                Some(x) => Ok(Box::new(File::open(x)?)),
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
    use clap::{App, AppSettings, Arg};
    use std::error::Error;
    use FileType::*;

    fn build_cli() -> App<'static, 'static> {
        let convert_types = [
            "y2j", "y2jp", "y2t", "y2tp", "j2y", "j2jp", "j2t", "j2tp", "t2y", "t2j", "t2jp",
            "t2tp",
        ];
        app_from_crate!()
            .setting(AppSettings::DeriveDisplayOrder)
            .arg(
                Arg::with_name("CONVERT_TYPE")
                    .possible_values(&convert_types)
                    .required(true),
            )
            .arg_from_usage("[FILENAME]")
    }

    fn convert_type(t: &str) -> ConvertType {
        match t {
            "y2j" => ConvertType::new(Yaml, Json, false),
            "y2jp" => ConvertType::new(Yaml, Json, true),
            "y2t" => ConvertType::new(Yaml, Toml, false),
            "y2tp" => ConvertType::new(Yaml, Toml, true),
            "j2y" => ConvertType::new(Json, Yaml, false),
            "j2jp" => ConvertType::new(Json, Json, true),
            "j2t" => ConvertType::new(Json, Toml, false),
            "j2tp" => ConvertType::new(Json, Toml, true),
            "t2y" => ConvertType::new(Toml, Yaml, false),
            "t2j" => ConvertType::new(Toml, Json, false),
            "t2jp" => ConvertType::new(Toml, Json, true),
            "t2tp" => ConvertType::new(Toml, Toml, true),
            _ => panic!("Unknown convert type"),
        }
    }

    fn parse_args() -> Result<(ConvertType, Option<String>), Box<dyn Error>> {
        let matches = build_cli().get_matches_safe()?;

        let cnvt_type = convert_type(matches.value_of("CONVERT_TYPE").unwrap());

        if let Some(filename) = matches.value_of("FILENAME") {
            Ok((cnvt_type, Some(String::from(filename))))
        } else {
            Ok((cnvt_type, None))
        }
    }

    fn convert() -> Result<String, Box<dyn Error>> {
        let (cnvt_type, input_stream) = parse_args()?;
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
