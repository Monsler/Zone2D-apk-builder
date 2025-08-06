use std::collections::HashMap;

use colored::Colorize;

use crate::log;



pub struct Builder {
    package_name: String,
    version: String,
    version_code: String,
    java_home: Option<String>,
}

impl Builder{
    pub fn new(properties: HashMap<&str, &str>) -> Self {
        return Self { package_name: properties.get("package_name").map_or("com.zone.app", |v| v).to_string(), version: properties.get("version_name").map_or("1.0.0", |v | v).to_string(), version_code: properties.get("version_code").map_or("10", |v | v).to_string(), java_home: properties.get("java_home").map(|s| s.to_string())};
    }

    fn apktool_work(&self) {
        log(crate::LogType::INFO, "Starting apktool");
    }

    pub fn build_to(&self, path: &str) {
        let java = self.java_home.clone().unwrap_or("java".to_string());
        
        log(crate::LogType::INFO, &format!("Package name: {}", self.package_name.white()));
        log(crate::LogType::INFO, &format!("Version: {}", self.version.white()));
        log(crate::LogType::INFO, &format!("Version Code: {}", self.version_code.white()));
        log(crate::LogType::INFO, &format!("Java Home: {}", java.white()).to_string());
        log(crate::LogType::WARN, "Starting build...");
        self.apktool_work();
    }
}

