use std::fs::File;
use std::path::Path;
use std::io;
use std::io::Read;
use std::io::Write;
use std::str;

use base64::encode;
use rpassword;
use yaml_rust::YamlLoader;

use error::{ErrorKind, Result};

// handle invalid configs by raising InvalidConfig if ever we try to get a value
// and it's not there
fn extract<F, T>(extractor: F) -> Result<T>
where
    F: Fn() -> Option<T>,
{
    match extractor() {
        Some(val) => Ok(val),
        None => Err(ErrorKind::InvalidConfig.into()),
    }
}

pub struct Defaults {
    pub project_key: String,
    pub assignee: String,
    pub labels: Vec<String>,
}

pub struct Config {
    pub jira_url: String,
    pub auth: String,
    pub username: String,
    pub projects: Vec<String>,
    pub npc_users: Vec<String>,
    pub open_in_browser: bool,
    pub browser_command: String,
    pub defaults: Defaults,
}

impl Config {
    pub fn new(path: &Path) -> Result<Config> {
        let mut file = try!(File::open(&path));
        let mut s = String::new();
        try!(file.read_to_string(&mut s));
        let docs = try!(YamlLoader::load_from_str(&s));
        let data = &docs[0];

        let jira_url = try!(extract(|| data["config"]["jira"].as_str())).to_string();
        let auth_data = try!(extract(|| data["config"]["auth"].as_str())).to_string();
        let username = try!(extract(|| data["config"]["username"].as_str())).to_string();

        let raw_projects = try!(extract(|| data["config"]["project_keys"].as_vec()));
        let mut projects = Vec::new();
        for elem in raw_projects.iter() {
            let val = try!(extract(|| elem.as_str())).to_string();
            projects.push(val);
        }

        let raw_npc_users = try!(extract(|| data["config"]["npc_users"].as_vec()));
        let mut npc_users = Vec::new();
        for elem in raw_npc_users.iter() {
            let val = try!(extract(|| elem.as_str())).to_string();
            npc_users.push(val);
        }
        let open_in_browser = try!(extract(|| data["config"]["open_in_browser"].as_bool()));
        let browser_command =
            try!(extract(|| data["config"]["browser_command"].as_str())).to_string();

        let default_project_key = try!(extract(|| data["config"]["new_issue_defaults"]
            ["project_key"]
            .as_str()))
            .to_string();
        let default_assignee = try!(extract(|| data["config"]["new_issue_defaults"]["assignee"]
            .as_str()))
            .to_string();
        let raw_default_labels = try!(extract(|| data["config"]["new_issue_defaults"]["labels"]
            .as_vec()));
        let mut default_labels = Vec::new();
        for elem in raw_default_labels {
            let val = try!(extract(|| elem.as_str())).to_string();
            default_labels.push(val);
        }

        Ok(Config {
            jira_url: jira_url,
            auth: auth_data,
            username: username,
            projects: projects,
            npc_users: npc_users,
            open_in_browser: open_in_browser,
            browser_command: browser_command,
            defaults: Defaults {
                project_key: default_project_key,
                assignee: default_assignee,
                labels: default_labels,
            },
        })
    }

    pub fn create(path: &Path) -> Result<Config> {
        print!("Jira url: ");
        try!(io::stdout().flush()); // need to do this since print! won't flush
        let mut jira = String::new();
        io::stdin().read_line(&mut jira).expect("Invalid jira url");

        print!("Username: ");
        try!(io::stdout().flush()); // need to do this since print! won't flush
        let mut username = String::new();
        io::stdin()
            .read_line(&mut username)
            .expect("Invalid username");

        let pass = rpassword::prompt_password_stdout("Password: ").unwrap();

        print!("Interrupt project key: ");
        try!(io::stdout().flush()); // need to do this since print! won't flush
        let mut project_key = String::new();
        io::stdin()
            .read_line(&mut project_key)
            .expect("Invalid project key");

        print!("Team username (a 'team' user like 'foo-robot'): ");
        try!(io::stdout().flush()); // need to do this since print! won't flush
        let mut npc = String::new();
        io::stdin()
            .read_line(&mut npc)
            .expect("Invalid team username");

        let auth = format!("{}:{}", username.trim(), pass.trim());
        let base64auth = encode(&auth);

        try!(create_config_file(
            path,
            jira.trim(),
            username.trim(),
            &base64auth,
            npc.trim(),
            project_key.trim()
        ));

        Config::new(path)
    }

    pub fn projects(&self) -> String {
        self.projects.join(", ")
    }

    pub fn npc_users(&self) -> String {
        self.npc_users.join(", ")
    }
}

fn create_config_file(
    path: &Path,
    jira: &str,
    username: &str,
    auth: &str,
    npc: &str,
    project_key: &str,
) -> Result<()> {
    let mut file = try!(File::create(&path));
    let content = format!(
        "# configuration for oh-bother
config_version: 1
config:
  # connectivity settings
  jira: \"{jira}\"
  username: \"{username}\"
  auth: \"{auth}\"

  # controls whether or not manipulated issues are opened in the web browser
  open_in_browser: true
  browser_command: google-chrome

  # These projects are used to find issues for commands like 'list' and 'next'
  project_keys:
    - \"{project_key}\"

  # these users are users for whom a ticket is considered 'fair game' or 'unassigned'
  npc_users:
    - Unassigned
    - \"{npc}\"

  new_issue_defaults:
    project_key: \"{project_key}\"
    assignee: \"{npc}\"
    labels:
      - interrupt
",
        jira = jira,
        username = username,
        auth = auth,
        npc = npc,
        project_key = project_key
    );

    try!(file.write_all(content.as_bytes()));
    Ok(())
}
