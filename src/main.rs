use std::{collections::HashMap, env};
use colored::Colorize;
use clap::Parser;
mod apk;

pub enum LogType {
    INFO,
    WARN,
    ERR,
    SUC
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

    #[arg(long = "keystore")]
    keystore: String,

    #[arg(long = "keystore-pwd")]
    keystore_pass: String,

    #[arg(long = "zpak")]
    zpak_path: String,

    #[arg(long = "javahome")]
    java_home: Option<String>,

    #[arg(long = "out")]
    out_path: Option<String>
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
        LogType::SUC => {
            println!("[S] {}", msg.green().bold());
        }
    }
}

fn main() {
    let mut properties: HashMap<&str, &str> = HashMap::new();
    
    let args = CliArgs::parse();
    properties.insert("package_name", &args.package_name);
    properties.insert("version_name", &args.version_name);
    properties.insert("version_code", &args.version_code);
    properties.insert("zpak_path", &args.zpak_path);
    properties.insert("keystore", &args.keystore);
    properties.insert("keystore_pass", &args.keystore_pass);

    let path = if let Some(out) = args.out_path {
        out
    } else {
        "out.apk".to_owned()
    };

    if let Some(java) = &args.java_home {
        properties.insert("java_home",java);
    }

    log(LogType::INFO,"Zone2D Builder CLI");
    log(LogType::INFO, &format!("Version: {}", VERSION));
    let builder = apk::Builder::new(properties);
    builder.build_to(&path);
    
}
