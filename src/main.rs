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

static ENTITY_SEARCH_DIRS: &'static [(&'static str, &'static str)] = &[
    ("MeeroApiBundle", "Meero\\ApiBundle\\Entity\\"),
    ("MeeroMediaBundle", "Meero\\MediaBundle\\ntity\\"),
    ("MeeroShootBundle", "Meero\\ShootBundle\\Entity\\"),
    ("MeeroShowcaseBundle", "Meero\\ShowcaseBundle\\Entity\\")
];

lazy_static! {
    /// Don't ever try to .write() this !
    static ref PROJECT_ROOT: RwLock<String> = RwLock::new(String::new());
    static ref WORK_DIR: RwLock<String> = RwLock::new(String::new());
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

    let project_root = {
        PROJECT_ROOT.write().unwrap().push_str(arg_matches.value_of("PROJECT_FD").unwrap());
        &PROJECT_ROOT.read().unwrap()
    };
    let work_dir = {
        WORK_DIR.write().unwrap().push_str(arg_matches.value_of("WORK_DIR").unwrap());
        &WORK_DIR.read().unwrap()
    };
    let controllers_conf = arg_matches.value_of("CONTROLLERS_CONF_YML").unwrap();

    let project_conf = format!("{}/app/config", project_root);
    let project_srcs = format!("{}/src/Meero", project_root);
    let symfony_srcs = format!("{}/vendor", project_root);

    let mut dealiaser = Dealiaser::new();
    let mut php = php::Php::new();

    f_find::f_find(&project_conf, ".yml", |s| dealiaser.clone().add_from_yml(s));
    f_find::f_find(&project_srcs, ".yml", |s| dealiaser.clone().add_from_yml(s));
    f_find::f_find(&symfony_srcs, ".php", |s| php.clone().add_from_php(s));
    f_find::f_find(&project_srcs, ".php", |s| php.clone().add_from_php(s));

    // php.rm_get(&dealiaser, work_dir as &str);
    php.rm_get_repository();
}
