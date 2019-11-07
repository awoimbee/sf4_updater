#![feature(try_trait)]
// FFS I KNOW WHAT I'M DOING. I'M USING A TRANSMUTE IN AN UNSAFE AND I STILL NEED THIS
#![allow(mutable_transmutes)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

mod conf;
mod dealiaser;
mod f_find;
mod php;

use conf::*;
use dealiaser::Dealiaser;
use f_find::f_find;

#[derive(Debug)]
pub struct Globals {
    pub project_root: String,
    pub project_conf: String,
    pub project_srcs: String,
    pub work_dir: String,
    pub controllers_yml: String,
    pub namespace_search_dirs: Vec<(String, String)>,
    pub entity_search_dirs: Vec<(String, String)>,
}

lazy_static! {
    static ref G: Globals = Globals::new();
}

// TODO: parse XML config
// TODO: add slog (https://github.com/slog-rs/term/blob/master/examples/compact-color.rs)
fn main() {
    let arg_matches = clap_app!(myapp =>
        (version: "0.2")
        (author: "Arthur W. <arthur.woimbee@gmail.com>")
        (about: "Helps you to update your sf3 project to sf4 & higher")
        (@arg PROJECT_FD: +takes_value --project_root -r "Path to your symfony project")
        (@arg CONTROLLERS_CONF_YML: +takes_value --controllers_yml -y "Path to file where controllers conf will be written")
        (@arg WORK_DIR: +takes_value --work_dir -w "Dir under which modifications will be done")
        (@arg CONF_FILE: +takes_value --conf_file -c "Conf. file to use")
        (@arg DEALIAS_REP: --dealias_getrepo -a "Transformer: dialias `getRopository()` statements")
        (@arg RM_GET: --rm_get -b "Transformer: remove `container->get()` statements")
    ).get_matches();

    let mut dealiaser = Dealiaser::new();
    let mut php = php::Php::new();

    load_conf(&arg_matches, &mut dealiaser);
    load_args(&arg_matches);

    let d = &mut dealiaser; // for `cargo fmt`
    f_find(&G.work_dir, ".php", |s| php.clone().add_from_php(s));
    f_find(&G.project_conf, ".yml", |s| d.clone().add_from_yml(s));
    f_find(&G.project_srcs, ".yml", |s| d.clone().add_from_yml(s));
    dealiaser.checkup();

    if arg_matches.is_present("DEALIAS_REP") {
        php.dealias_get_repository();
    }
    if arg_matches.is_present("RM_GET") {
        php.rm_get(&dealiaser);
    }
}
