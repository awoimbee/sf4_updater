#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
// extern crate yaml_rust;

mod dealiaser;
mod f_find;
mod php;

use dealiaser::Dealiaser;
use std::sync::RwLock;

static NAMESPACE_SEARCH_DIRS: &'static [&'static str] =
    &["/src/Meero/", "/vendor/", "/src/Meero/DataFixtures/"];

lazy_static! {
    /// Don't ever try to .write() this !
    static ref PROJECT_ROOT: RwLock<String> = RwLock::new(String::new());
}

// TODO: parse XML config
fn main() {
    let arg_matches = clap_app!(myapp =>
        (version: "0.1")
        (author: "Arthur W. <arthur.woimbee@gmail.com>")
        (about: "Does awesome things")
        (@arg PROJECT_FD: +required "Sets where to look for php and yml files")
        (@arg CONTROLLERS_CONF_YML: +required "Sets where to write controller service declarations")
        (@arg WORK_DIR: +required "Sets where to update php files")
        // (@arg debug: -d ... "Sets the level of debugging information")
    )
    .get_matches();

    let mut proot = PROJECT_ROOT.write().unwrap();
    proot.push_str(arg_matches.value_of("PROJECT_FD").unwrap());
    drop(proot);

    let project_root = PROJECT_ROOT.read().unwrap();
    let controllers_conf = arg_matches.value_of("CONTROLLERS_CONF_YML").unwrap();
    let work_dir = arg_matches.value_of("WORK_DIR").unwrap();

    let project_conf = format!("{}/app/config", project_root);
    let project_srcs = format!("{}/src/Meero", project_root);
    let symfony_srcs = format!("{}/vendor", project_root);

    let mut dealiaser = Dealiaser::new();
    let mut php = php::Php::new();

    f_find::f_find(&project_conf, ".yml", |s| dealiaser.clone().add_from_yml(s));
    f_find::f_find(&project_srcs, ".yml", |s| dealiaser.clone().add_from_yml(s));
    f_find::f_find(&symfony_srcs, ".php", |s| php.clone().add_from_php(s));
    f_find::f_find(&project_srcs, ".php", |s| php.clone().add_from_php(s));

    php::resolve_namespace::resolve_namespace("tut");

    php.rm_get(&dealiaser, work_dir);
    // println!("{:?}\n\n\n\n", dealiaser);
    // println!("{:?}", php);
}
