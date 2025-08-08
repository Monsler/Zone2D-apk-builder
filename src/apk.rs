use std::{collections::HashMap, env, fmt::format, fs, path::PathBuf, process::{exit, Command}, time::{SystemTime, UNIX_EPOCH}};
use regex::Regex;
use colored::Colorize;
use crate::log;
use crate::LogType;
pub struct Builder {
    package_name: String,
    version: String,
    version_code: String,
    java_home: Option<String>,
    sign_key: String,
    zpak_path: String,
    sign_key_pass: String
}

struct CommandBuilder {
}

impl CommandBuilder {
    pub fn new() -> Self {
        return Self{};
    }

    pub fn make_apktool_command(&self, output: String) -> Vec<String> {
        let mut cmd: Vec<String> = Vec::new();
        let template_dir = PathBuf::from("template");
        cmd.push("-jar".to_string());
        cmd.push("apktool.jar".to_string());
        cmd.push("b".to_string());
        let result = dunce::canonicalize(&template_dir);
        match result {
            Ok(result) => {
                match result.to_str() {
                    Some(result) => {
                        cmd.push(result.to_string());
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

        
        cmd.push("-o".to_string());
        cmd.push(output.to_string());

    
        return cmd;
    }

    pub fn make_sign_command(&self, keystore_path: String, keystore_pass: String, input: String, output: String) -> Vec<String> {
        let mut command: Vec<String> = Vec::new();

        command.push("-jar".to_owned());
        command.push("apksigner.jar".to_owned());
        command.push("sign".to_owned());
        command.push("--ks".to_owned());
        command.push(keystore_path);
        command.push("--ks-pass".to_owned());
        command.push(format!("pass:{}", keystore_pass));
        command.push("--key-pass".to_owned());
        command.push(format!("pass:{}", keystore_pass));
        command.push("--out".to_owned());
        command.push(output);
        command.push(input);

        return command;
    }

    pub fn make_zipalign_command(&self, input: String, output: String) -> Vec<String> {
        let mut command: Vec<String> = Vec::new(); 
        if fs::exists(&output).unwrap() {
            let _ = fs::remove_file(&output);
        }
        let current_os = std::env::consts::OS;
        let mut path = match dunce::canonicalize("buildtools") {
            Ok(abs_path) => {
               abs_path
            },
            Err(error) => {
                log(LogType::ERR, &format!("Error canonicalizing relative path to the build tools: {}", error));
                exit(-1);
            }
        };

        path.push(&current_os);
        path.push("zipalign");
        if current_os == "windows" {
            path.set_extension("exe");
        }

        match path.to_str() {
            Some(path_str) => {
                command.push(path_str.to_string());
            },
            None => {
                log(LogType::ERR, "path.to_str() is none");
                exit(-1);
            }
        }
        command.push("-v".to_owned());
        command.push("4".to_owned());
        command.push(input);
        command.push(output);


        return command;
    }
}

impl Builder {
    pub fn new(properties: HashMap<&str, &str>) -> Self {
        Self { package_name: properties.get("package_name").map_or("com.zone.app", |v| v).to_string(), version: properties.get("version_name").map_or("1.0.0", |v | v).to_string(), version_code: properties.get("version_code").map_or("10", |v | v).to_string(), java_home: properties.get("java_home").map(|s| s.to_string()),
                    zpak_path: properties.get("zpak_path").map_or("resource.zpak", |v | v).to_string(), sign_key: properties.get("keystore").map_or("android.keystore", |v | v).to_string(), sign_key_pass: properties.get("keystore_pass").map_or("", | v | v).to_string()}
    }

    fn edit_manifest(&self) {
        log(crate::LogType::INFO, "Editing manifest...");
        match  fs::read_to_string("template/AndroidManifest.xml") {
            Ok(code) => {
               let regex = Regex::new(r#"(package|android:versionCode|android:versionName|android:authorities)="[^"]+""#).unwrap();
               let modified = regex.replace_all(&code, |caps: &regex::Captures | {
                   match &caps[1] {
                       "package" => format!("package=\"{}\"", &self.package_name).to_string(),
                       "android:versionCode" => format!("android:versionCode=\"{}\"", self.version_code).to_string(),
                       "android:versionName" => format!("android:versionName=\"{}\"", self.version).to_string(),
                       "android:authorities" => format!("android:authorities=\"{}.androidx-startup\"", self.package_name).to_string(),
                       _ => caps[0].to_string(),
                   }
               });
                if let Err(error) = fs::write("template/AndroidManifest.xml", modified.as_bytes()) {
                    log(LogType::ERR, &format!("Error writing manifest: {}", error));
                    exit(-1);
                }
            },
            Err(error) => {
                log(crate::LogType::ERR, &format!("Error reading manifest: {}", error));
                exit(-1);
            }
        }
        log(crate::LogType::INFO, "Manifest modified successfully");
    }

    fn apksigner_work(&self, command: Vec<String>, java_home: &String) -> i32 {
        log(LogType::WARN, "Signing apk...");
        
        let mut str_command = String::new();
        for elem in &command {
            str_command.push_str(" ");
            str_command.push_str(&elem);
        }
        log(crate::LogType::INFO,  &format!("Running command: {}{}", &java_home, str_command));

        let mut process = Command::new(java_home);
        process.args(&command);

        match process.output() {
            Ok(output) => {
                let status_code = output.status.code().unwrap();
                let stdout = output.stdout;
                let stderr = output.stderr;
                log(crate::LogType::INFO, &format!("Program finished with code {}", status_code));
                log(crate::LogType::INFO, &format!("Output:\n{}\n{}", String::from_utf8(stdout).unwrap(), String::from_utf8(stderr).unwrap().red()));
                return status_code;
            },
            Err(error) => {
                log(crate::LogType::ERR, &format!("Error: {}", error));
                return -1;
            }
        }
    }

    fn apktool_work(&self, command: Vec<String>, java_home: &String) -> i32 {
   
        log(crate::LogType::INFO, "Starting apktool");
        let mut str_command = String::new();
        for elem in &command {
            str_command.push_str(" ");
            str_command.push_str(&elem);
        }
        log(crate::LogType::INFO,  &format!("Running command: {}{}", &java_home, str_command));

        let mut cmd = Command::new(java_home);
        cmd.args(&command);
        match cmd.output() {
            Ok(output) => {
                let status_code = output.status.code().unwrap();
                let stdout = output.stdout;
                let stderr = output.stderr;
                log(crate::LogType::INFO, &format!("Program finished with code {}", status_code));
                log(crate::LogType::INFO, &format!("Output:\n{}\n{}", String::from_utf8(stdout).unwrap(), String::from_utf8(stderr).unwrap().red()));
                return status_code;
            }
            Err(error) => {
                log(crate::LogType::ERR, &format!("Error: {}", error));
                return -1;
            }
        }
    }
    
    fn zipalign_work(&self, command: Vec<String>) -> i32 {
        log(LogType::WARN, "Aligning apk...");
        let mut str_command = String::new();
        let process = &command[0];
        for elem in &command {
            str_command.push_str(" ");
            str_command.push_str(&elem);
        }
        log(crate::LogType::INFO,  &format!("Running command: {}", str_command));
        
        let mut cmd = Command::new(&process);
        cmd.args(&command[1..]);

        match cmd.output() {
            Ok(output) => {
                let status_code = output.status.code().unwrap();
                let stdout = output.stdout;
                let stderr = output.stderr;
                log(LogType::INFO, &format!("Program finished with code {}", status_code));
                log(LogType::INFO, &format!("Output:\n{}\n{}", String::from_utf8(stdout).unwrap(), String::from_utf8(stderr).unwrap().red()));
                return status_code;
            },
            Err(error) => {
                log(LogType::ERR, &format!("Error: {}", error));
                exit(-1);
            }
         }
    } 
    
    pub fn build_to(&self, path: &str) {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut java = String::new();
        let zpak_dest = "template/assets/resource.zpak";
        if let Some(home) = &self.java_home {
            java.push_str(&home);
            java.push_str("/bin/java");
            if std::env::consts::OS == "windows" {
                java.push_str(".exe");
            }
        } else {
            match env::var("JAVA_HOME") {
                Ok(path) => {
                    java.push_str(&path);
                    java.push_str("/bin/java");
                    if std::env::consts::OS == "windows" {
                        java.push_str(".exe");
                    }
                },
                Err(error) => {
                    log(crate::LogType::ERR, &format!("Error getting JAVA_HOME: {}", error.to_string()));
                }
            }
           
        }
        if fs::exists(zpak_dest).unwrap() {
           let _ = fs::remove_file(zpak_dest);
        }
        if !fs::exists(&self.zpak_path).unwrap() {
            log(LogType::ERR, &format!("Zpak cannot be found at path you specified: {}", &self.zpak_path));
            exit(-1);
        }
        if fs::exists("template/build").unwrap() {
            let _ = fs::remove_dir_all("template/build");
        }


        log(LogType::INFO, &format!("Copying {} to {}", &self.zpak_path, zpak_dest));
        let _ = fs::copy(&self.zpak_path, zpak_dest);
        let command_builder = CommandBuilder::new();
        log(crate::LogType::INFO, &format!("Package name: {}", self.package_name.white()));
        log(crate::LogType::INFO, &format!("Version: {}", self.version.white()));
        log(crate::LogType::INFO, &format!("Version Code: {}", self.version_code.white()));
        log(crate::LogType::INFO, &format!("Java Home: {}", java.white()).to_string());
        log(crate::LogType::WARN, "Starting build...");
        self.edit_manifest();
        let result = self.apktool_work(command_builder.make_apktool_command("base.apk".to_owned()), &java);
        if result == 0 {
            let zipalign_result = self.zipalign_work(command_builder.make_zipalign_command("base.apk".to_owned(), "base_aligned.apk".to_owned()));
            if zipalign_result == 0 {
                log(LogType::SUC, "Aligned successfully");
                let sign_result = self.apksigner_work(command_builder.make_sign_command(self.sign_key.to_owned(), self.sign_key_pass.to_owned(), "base_aligned.apk".to_owned(), path.to_owned()), &java);
                if sign_result == 0 {
                    log(LogType::SUC, &format!("Built successfully. Saved as {}", path));
                    let new_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    let cost = new_time - current_time;
                    log(LogType::INFO, &format!("Spent {} second(-s)", cost));
                }
            }
        }
    }
}

