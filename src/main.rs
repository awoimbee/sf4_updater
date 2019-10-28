#[macro_use]
extern crate clap;
// extern crate colored;
extern crate yaml_rust;

mod f_find;
mod dealiaser;

fn l(s: &str) {
    println!("{}", s);
}

use dealiaser::Dealiaser;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let arg_matches = clap_app!(myapp =>
        (version: "0.1")
        (author: "Arthur W. <arthur.woimbee@gmail.com>")
        (about: "Does awesome things")
        (@arg PROJECT_FD: +required "Sets where to look for php and yml files")
        (@arg CONTROLLERS_CONF_YML: +required "Sets where to write controller service declarations")
        // (@arg debug: -d ... "Sets the level of debugging information")
    ).get_matches();

    let project_root = arg_matches.value_of("PROJECT_FD").unwrap();
    let project_conf = format!("{}/app/config", project_root);
    let project_srcs = format!("{}/src/Meero", project_root);

    let mut dealiaser = Dealiaser::new();

    f_find::f_find(&project_conf, ".yml", |f| dealiaser.clone().add_from_yml(f));
    f_find::f_find(&project_srcs, ".yml", |f| dealiaser.clone().add_from_yml(f));
    f_find::f_find(&project_srcs, ".php", l);

    println!("{:?}", dealiaser);
}
