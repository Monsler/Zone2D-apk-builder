use std::{collections::HashMap, env};
use colored::Colorize;
use clap::Parser;
mod apk;

pub enum LogType {
    INFO,
    WARN,
    ERR
}

static VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(long = "package")]
    package_name: String,

    #[arg(long = "version")]
    version_name: String,

    #[arg(long = "versioncode")]
    version_code: String,

    #[arg(long = "javahome")]
    java_home: Option<String>,
}

pub fn log(msg_type: LogType, msg: &str) {
    match msg_type {
        LogType::INFO => {
            println!("[I] {}", msg.blue());
        },
        LogType::WARN => {
            println!("[W] {}", msg.yellow());
        },
        LogType::ERR => {
            println!("[E] {}", msg.red());
        },
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut properties: HashMap<&str, &str> = HashMap::new();
    let mut path = "./base.apk";
    let args = CliArgs::parse();
    properties.insert("package_name", &args.package_name);
    properties.insert("version_name", &args.version_name);
    properties.insert("version_code", &args.version_code);
    if let Some(java) = &args.java_home {
        properties.insert("java_home",java);
    }


    log(LogType::INFO,"Zone2D Builder CLI");
    log(LogType::INFO, &format!("Version: {}", VERSION));
    let builder = apk::Builder::new(properties);
    builder.build_to(path);
    
}
