#![feature(try_trait)]
#![feature(const_fn)]
#![feature(unsized_locals)]
#![allow(mutable_transmutes)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

mod conf;
mod f_find;
mod php;

use conf::*;
use f_find::f_find;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Globals {
    pub root_namespace: String,
    pub project_root: String,
    pub project_conf: String,
    pub project_srcs: String,
    pub work_dir: String,
    pub controllers_yml: String,
    pub namespace_search_dirs: Vec<(String, String)>,
    pub entity_search_dirs: Vec<(String, String)>,
    pub dealiaser_additionals: Vec<(String, String)>,
    pub bundles: Vec<(String, String)> // (name, path)
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
        (@arg DEALIAS_REP: --dealias_getrepo -A "Transformer: dialias `getRopository()` statements")
        (@arg RM_GET: --rm_get -B "Transformer: remove `container->get()` statements")
        (@arg DEALIAS_PATHS: --dealias_paths -C "Transformer: remove path aliases")
    ).get_matches();


    let php = Arc::new(Mutex::new(php::Php::new()));
    let mut php_w = php.lock().unwrap();

    load_conf(&arg_matches);
    load_args(&arg_matches);


    f_find(&G.work_dir, ".php", |s| drop(php_w.add_from_php(s)));



    if arg_matches.is_present("DEALIAS_REP") {
        php_w.dealias_get_repository();
    }
    if arg_matches.is_present("RM_GET") {
        php_w.rm_get();
    }
    if arg_matches.is_present("DEALIAS_PATHS") {
        php_w.dealias_paths();
    }
}
