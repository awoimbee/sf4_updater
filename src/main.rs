#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
// extern crate yaml_rust;

mod dealiaser;
mod f_find;
mod php;

use dealiaser::Dealiaser;

fn main() {
    let arg_matches = clap_app!(myapp =>
        (version: "0.1")
        (author: "Arthur W. <arthur.woimbee@gmail.com>")
        (about: "Does awesome things")
        (@arg PROJECT_FD: +required "Sets where to look for php and yml files")
        (@arg CONTROLLERS_CONF_YML: +required "Sets where to write controller service declarations")
        // (@arg debug: -d ... "Sets the level of debugging information")
    )
    .get_matches();

    let project_root = arg_matches.value_of("PROJECT_FD").unwrap();
    let project_conf = format!("{}/app/config", project_root);
    let project_srcs = format!("{}/src/Meero", project_root);
    let symfony_srcs = format!("{}/vendor", project_root);

    let mut dealiaser = Dealiaser::new();
    let mut php = php::Php::new();

    f_find::f_find(&project_conf, ".yml", |s| dealiaser.clone().add_from_yml(s));
    f_find::f_find(&project_srcs, ".yml", |s| dealiaser.clone().add_from_yml(s));
    f_find::f_find(&symfony_srcs, ".php", |s| php.clone().add_from_php(s));
    f_find::f_find(&project_srcs, ".php", |s| php.clone().add_from_php(s));

    php.rm_get("");
    // println!("{:?}\n\n\n\n", dealiaser);
    // println!("{:?}", php);
}
