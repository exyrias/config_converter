#[macro_use]
extern crate clap;

mod converter {
    extern crate serde;
    extern crate serde_derive;
    extern crate serde_json;
    extern crate serde_yaml;
    extern crate toml;

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
            ConvertType::Yaml2Json => serde_json::to_string(&load_yaml(&data)?)?,
            ConvertType::Yaml2JsonPretty => serde_json::to_string_pretty(&load_yaml(&data)?)?,
            ConvertType::Json2Yaml => serde_yaml::to_string(&load_json(&data)?)?,
            ConvertType::Json2JsonPretty => serde_json::to_string_pretty(&load_json(&data)?)?,
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

    fn build_cli() -> App<'static, 'static> {
        app_from_crate!()
            .setting(AppSettings::DeriveDisplayOrder)
            .arg(
                Arg::with_name("CONVERT_TYPE")
                    .possible_values(&["y2j", "y2jp", "j2y", "j2jp"])
                    .required(true),
            )
            .arg_from_usage("[FILENAME]")
    }

    fn convert_type(t: &str) -> ConvertType {
        match t {
            "y2j" => ConvertType::Yaml2Json,
            "y2jp" => ConvertType::Yaml2JsonPretty,
            "j2y" => ConvertType::Json2Yaml,
            "j2jp" => ConvertType::Json2JsonPretty,
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
