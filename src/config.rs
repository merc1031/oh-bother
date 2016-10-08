use rpassword;
use yaml_rust::YamlLoader;
use yaml_rust::scanner::ScanError;

use std::fs::File;
use std::path::Path;
use std::fmt;
use std::io;
use std::io::Read;
use std::io::Write;

use rustc_serialize::base64::{ToBase64, STANDARD};

pub struct Config {
    pub jira_url: String,
    pub auth: String,
    pub username: String,
}

type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    ParseError(ScanError),
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::IoError(err)
    }
}

impl From<ScanError> for ConfigError {
    fn from(err: ScanError) -> ConfigError {
        ConfigError::ParseError(err)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::IoError(ref e) => e.fmt(f),
            ConfigError::ParseError(ref e) => e.fmt(f),
        }
    }
}

impl Config {
    pub fn new(path: &Path) -> ConfigResult<Config> {
        let mut file = try!(File::open(&path));
        let mut s = String::new();
        try!(file.read_to_string(&mut s));
        let docs = try!(YamlLoader::load_from_str(&s));
        let data = &docs[0];
        Ok(Config {
            jira_url: data["config"]["jira"].as_str().unwrap().to_string(),
            auth: data["config"]["auth"].as_str().unwrap().to_string(),
            username: data["config"]["username"].as_str().unwrap().to_string(),
        })
    }

    pub fn create(path: &Path) -> ConfigResult<Config> {
        print!("Jira url: ");
        try!(io::stdout().flush()); // need to do this since print! won't flush
        let mut jira = String::new();
        io::stdin().read_line(&mut jira).expect("Invalid jira url");

        print!("Username: ");
        try!(io::stdout().flush()); // need to do this since print! won't flush
        let mut username = String::new();
        io::stdin().read_line(&mut username).expect("Invalid username");

        print!("Interrupt project key: ");
        try!(io::stdout().flush()); // need to do this since print! won't flush
        let mut project_key = String::new();
        io::stdin().read_line(&mut project_key).expect("Invalid project key");

        let pass = rpassword::prompt_password_stdout("Password: ").unwrap();
        let auth = format!("{}:{}", username.trim(), pass.trim());
        let base64auth = auth.as_bytes().to_base64(STANDARD);

        try!(create_config_file(path,
                                &jira.trim(),
                                username.trim(),
                                &base64auth,
                                &project_key.trim()));

        Config::new(path)
    }
}

fn create_config_file(path: &Path,
                      jira: &str,
                      username: &str,
                      auth: &str,
                      project_key: &str)
                      -> Result<(), io::Error> {
    let mut file = try!(File::create(&path));
    let content = format!("# configuration for oh-bother
config_version: 1
config:
  # connectivity settings
  jira: \"{}\"
  username: \"{}\"
  auth: \"{}\"

  # controls whether or not manipulated issues are opened in the web browser
  open_in_browser: true
  browser_command: google-chrome

  interrupt_defaults:
    project_key: {}
    labels:
      - interrupt
",
                          jira,
                          username,
                          auth,
                          project_key);

    try!(file.write_all(content.as_bytes()));
    Ok(())
}
