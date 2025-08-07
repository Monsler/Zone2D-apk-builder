use std::{collections::HashMap, fs, path::PathBuf, process::exit, result};

use colored::Colorize;

use crate::log;



pub struct Builder {
    package_name: String,
    version: String,
    version_code: String,
    java_home: Option<String>,
}

struct CommandBuilder {
    java_home: String
}

impl CommandBuilder {
    pub fn new(java_home: &str) -> Self {
        return Self{java_home: java_home.to_string()};
    }

    pub fn make_apktool_command(&self, output: &str) -> String {
        let mut cmd = String::new();
        let template_dir = PathBuf::from("template");
        cmd.push_str(&self.java_home);
        cmd.push_str(" -jar ");
        cmd.push_str(" apktool.jar ");
        cmd.push_str(" b ");
        let result = dunce::canonicalize(template_dir);
        match result {
            Ok(result) => {
                match result.to_str() {
                    Some(result) => {
                        cmd.push_str(result);
                    }
                    None => {
                        log(crate::LogType::ERR, "Pizdec");
                        exit(-1);
                    }
                }
            },
            Err(error) => {
                log(crate::LogType::ERR, &format!("Unable to canonicalize the apk template path: {}", error.to_string()));
                exit(-1);
            },
        }
        
        cmd.push_str(" -o ");
        cmd.push_str(output);
        return cmd;
    }
}

impl Builder {
    pub fn new(properties: HashMap<&str, &str>) -> Self {
        return Self { package_name: properties.get("package_name").map_or("com.zone.app", |v| v).to_string(), version: properties.get("version_name").map_or("1.0.0", |v | v).to_string(), version_code: properties.get("version_code").map_or("10", |v | v).to_string(), java_home: properties.get("java_home").map(|s| s.to_string())};
    }

    fn apktool_work(&self, command: &str) {
   
        log(crate::LogType::INFO, "Starting apktool");
        log(crate::LogType::INFO,  &format!("Running command: {}", command));
    }

    pub fn build_to(&self, path: &str) {
        let java = self.java_home.clone().unwrap_or("java".to_string());
        let command_builder = CommandBuilder::new(&java);
        log(crate::LogType::INFO, &format!("Package name: {}", self.package_name.white()));
        log(crate::LogType::INFO, &format!("Version: {}", self.version.white()));
        log(crate::LogType::INFO, &format!("Version Code: {}", self.version_code.white()));
        log(crate::LogType::INFO, &format!("Java Home: {}", java.white()).to_string());
        log(crate::LogType::WARN, "Starting build...");
        self.apktool_work(&command_builder.make_apktool_command("base.apk"));
    }
}

