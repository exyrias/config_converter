
mod cli {
    use clap::{Parser, ValueEnum};
    #[derive(Parser, Debug, Clone)]
    pub struct Args {
        pub convert_type: CmdConvertType,
        pub input_file: Option<String>,
    }

    #[derive(ValueEnum, Debug, Clone, Copy)]
    pub enum CmdConvertType {
        #[clap(name = "yj")]
        YamlToJson,
        #[clap(name = "yjp")]
        YamlToJsonPretty,
        #[clap(name = "yt")]
        YamlToToml,
        #[clap(name = "ytp")]
        YamlToTomlPretty,
        #[clap(name = "jy")]
        JsonToYaml,
        #[clap(name = "jjp")]
        JsonToJsonPretty,
        #[clap(name = "jt")]
        JsonToToml,
        #[clap(name = "jtp")]
        JsonToTomlPretty,
        #[clap(name = "ty")]
        TomlToYaml,
        #[clap(name = "tj")]
        TomlToJson,
        #[clap(name = "tjp")]
        TomlToJsonPretty,
        #[clap(name = "ttp")]
        TomlToTomlPretty,
        #[clap(name = "y")]
        Yaml,
        #[clap(name = "j")]
        Json,
        #[clap(name = "jp")]
        JsonPretty,
        #[clap(name = "t")]
        Toml,
        #[clap(name = "tp")]
        TomlPretty,
    }

}

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

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum FileType {
        Yaml,
        Json,
        Toml,
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct ConvertType {
        from: Option<FileType>,
        to: FileType,
        prettify: bool,
    }
    impl ConvertType {
        pub fn new(from: Option<FileType>, to: FileType, prettify: bool) -> Self {
            Self { from, to, prettify }
        }
    }

    pub fn convert_string(
        cnvt_type: ConvertType,
        data: &String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let v = if let Some(type_from) = cnvt_type.from {
            match type_from {
                FileType::Yaml => load_yaml(&data)?,
                FileType::Json => load_json(&data)?,
                FileType::Toml => load_toml(&data)?,
            }
        } else {
            let v = load_yaml(&data);
            if v.is_ok() {
                v.unwrap()
            } else {
                let v = load_json(&data);
                if v.is_ok() {
                    v.unwrap()
                } else {
                    load_toml(&data)?
                }
            }
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
    use crate::cli::*;
    use std::error::Error;
    use clap::Parser;
    use FileType::*;

    fn convert_type(t: CmdConvertType) -> ConvertType {
        match t {
            CmdConvertType::YamlToJson => ConvertType::new(Some(Yaml), Json, false),
            CmdConvertType::YamlToJsonPretty => ConvertType::new(Some(Yaml), Json, true),
            CmdConvertType::YamlToToml => ConvertType::new(Some(Yaml), Toml, false),
            CmdConvertType::YamlToTomlPretty => ConvertType::new(Some(Yaml), Toml, true),
            CmdConvertType::JsonToYaml => ConvertType::new(Some(Json), Yaml, false),
            CmdConvertType::JsonToJsonPretty => ConvertType::new(Some(Json), Json, true),
            CmdConvertType::JsonToToml => ConvertType::new(Some(Json), Toml, false),
            CmdConvertType::JsonToTomlPretty => ConvertType::new(Some(Json), Toml, true),
            CmdConvertType::TomlToYaml => ConvertType::new(Some(Toml), Yaml, false),
            CmdConvertType::TomlToJson => ConvertType::new(Some(Toml), Json, false),
            CmdConvertType::TomlToJsonPretty => ConvertType::new(Some(Toml), Json, true),
            CmdConvertType::TomlToTomlPretty => ConvertType::new(Some(Toml), Toml, true),
            CmdConvertType::Yaml => ConvertType::new(None, Yaml, false),
            CmdConvertType::Json => ConvertType::new(None, Json, false),
            CmdConvertType::JsonPretty => ConvertType::new(None, Json, true),
            CmdConvertType::Toml => ConvertType::new(None, Toml, false),
            CmdConvertType::TomlPretty => ConvertType::new(None, Toml, true),
        }
    }

    fn convert() -> Result<String, Box<dyn Error>> {
        let args = Args::parse();
        let cnvt_type = convert_type(args.convert_type);
        let mut input_stream = read_utils::input_selector(args.input_file)?;
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
