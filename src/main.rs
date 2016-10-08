extern crate clap;
extern crate url;
extern crate hyper;
extern crate rustc_serialize;
extern crate rpassword;
extern crate yaml_rust;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use hyper::Url;
use hyper::Client;
use hyper::client::IntoUrl;
use hyper::header::{Headers, Authorization, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

use rustc_serialize::{Encodable, json};

use std::path::Path;
use std::env;
use std::io::Read;

use config::Config;

mod config;
mod jira;
mod util;

fn main() {
    let default_config_path = env::home_dir().unwrap().join(".ob.yml");
    let matches = App::new("ob")
        .version("1.0.0")
        .author("Matt Chun-Lum")
        .about("JIRA interrupt management")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .arg(Arg::with_name("dry")
            .help("show what request(s) would be made")
            .long("dry-run"))
        .arg(Arg::with_name("config")
            .help("sets the config file to use")
            .takes_value(true)
            .default_value(default_config_path.to_str().unwrap())
            .short("c")
            .long("config"))
        .subcommand(SubCommand::with_name("setup")
            .about("Configures oh-bother")
            .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("list")
            .about("Lists interrupts")
            .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("start")
            .about("Start the specified interrupt")
            .arg(Arg::with_name("issue")
                .help("the issue")
                .index(1)
                .required(true))
            .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("close")
            .about("Close the specified interrupt")
            .arg(Arg::with_name("issue")
                .help("the issue")
                .index(1)
                .required(true))
            .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("new")
            .about("Creates a new interrupt")
            .arg(Arg::with_name("title")
                .help("the title of the interrupt")
                .index(1)
                .required(true))
            .arg(Arg::with_name("foo")
                .help("Foo")
                .long("foo"))
            .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("jql")
            .about("Execute a raw jql query")
            .arg(Arg::with_name("query")
                .help("the jql query")
                .index(1)
                .required(true))
            .setting(AppSettings::ColoredHelp))
        .get_matches();

    let config_file = matches.value_of("config").unwrap();
    let config_path = Path::new(config_file);

    if matches.is_present("setup") {
        match Config::create(&config_path) {
            Err(why) => {
                println!("There was an error creating the config.");
                util::exit(&format!("couldn't create config file {}: {}",
                                    config_path.display(),
                                    why))
            }
            Ok(_) => {}
        }
        println!("Please edit {} to include your desired configuration",
                 config_path.display());
    } else {
        let config = match Config::new(&config_path) {
            Err(why) => {
                println!("There was an error loading the config. Maybe run 'setup'?");
                util::exit(&format!("couldn't open config file {}: {}",
                                    config_path.display(),
                                    why))
            }
            Ok(config) => config,
        };

        match matches.subcommand_name() {
            Some("list") => println!("list"),
            Some("start") => println!("start"),
            Some("close") => println!("close"),
            Some("new") => println!("new"),
            Some("jql") => jql(&config, &matches),
            _ => util::exit("unknown command"), // shouldn't really ever get here
        }
    }
}

fn jql(conf: &Config, matches: &ArgMatches) {
    let subcmd = match matches.subcommand_matches("jql") {
        Some(matches) => matches,
        None => util::exit("wtf")
    };

    let query = subcmd.value_of("query").unwrap();

    let mut headers = Headers::new();
    headers.set(Authorization(format!("Basic {}", conf.auth).to_owned()));
    headers.set(ContentType(Mime(TopLevel::Application,
                                 SubLevel::Json,
                                 vec![(Attr::Charset, Value::Utf8)])));
    let client = Client::new();
    let base_url = Url::parse(conf.jira_url.as_str()).unwrap();
    let mut url = base_url.join("rest/api/2/search").unwrap();
    let foo = JQLQuery {
        jql: query.to_string(),
        fields: vec!["summary".to_string(), "status".to_string(), "assignee".to_string()],
    };
    let body = json::encode(&foo).unwrap();
    let mut res = client.post(url).headers(headers).body(body.as_str()).send().unwrap();
    let mut resp = String::new();
    match res.read_to_string(&mut resp) {
        Err(why) => util::exit(&format!("Error! {}", why)),
        Ok(_) => {}
    }
    println!("{}", resp)

}
