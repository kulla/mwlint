extern crate mediawiki_parser;
extern crate serde_yaml;
extern crate argparse;
extern crate mwlint;
extern crate toml;

use std::process;
use argparse::{ArgumentParser, StoreTrue, Store};

macro_rules! DESCRIPTION {
() => (
"This program takes a yaml syntax tree of a mediawiki document
(as created by `mwtoast`) as input and checks it for for discouraged
patterns and other nitpicks."
)
}

fn main() {
    let mut use_stdin = false;
    let mut dump_config = false;
    let mut input_file = "".to_string();
    let mut config_file = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description(DESCRIPTION!());
        ap.refer(&mut use_stdin).add_option(
            &["-s", "--stdin"],
            StoreTrue,
            "Use stdin as input file",
        );

        ap.refer(&mut input_file).add_option(
            &["-i", "--input"],
            Store,
            "Path to the input file",
        );
        ap.refer(&mut dump_config).add_option(
            &["-d", "--dump-settings"],
            StoreTrue,
            "Dump the current settings to stdout. \
             If no configuration file is loaded, \
             dump the default options."
        );
        ap.refer(&mut config_file).add_option(
            &["-c", "--config"],
            Store,
            "A config file to override the default options."
        );
        ap.parse_args_or_exit();
    }
    let root;

    let mut settings: mwlint::settings::Settings;

    if !config_file.is_empty() {
        let config_source = mediawiki_parser::util::read_file(&config_file);
        settings = toml::from_str(&config_source)
            .expect("Could not parse settings file!");
    } else {
        settings = Default::default();
    }

    settings.merge_rules(&mut mwlint::rules::get_rules());

    if dump_config {
        println!("{}", toml::to_string(&settings)
            .expect("Could serialize settings!"));
        process::exit(0);
    }

    let input: String;
    if use_stdin {
        input = mediawiki_parser::util::read_stdin();
    } else if !input_file.is_empty() {
        input = mediawiki_parser::util::read_file(&input_file);
    } else {
        eprintln!("No input source specified!");
        process::exit(1);
    }


    root= serde_yaml::from_str(&input).expect("Could not parse input file!");
    let mut lints = vec![];
    let mut path = vec![];

    for rule in &settings.rules[..] {
        if rule.enable {
            match rule.function {
                Some(f) => f(&root, &mut path, &settings, &mut lints),
                None => (),
            };
        }
    }

    for lint in &lints {
        eprintln!("{}", lint);
    }
    println!("{}", serde_yaml::to_string(&lints).expect("Could not serialize lint!"));

}
