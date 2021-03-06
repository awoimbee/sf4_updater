use crate::Globals;
use colored::*;
use std::fs::File;
use std::io::prelude::*;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

const DEFAULT_CONF_FILE: &str = "./config.yml";

impl Globals {
    pub fn new() -> Globals {
        Globals {
            project_root: String::new(),
            project_conf: String::new(),
            project_srcs: String::new(),
            work_dir: String::new(),
            controllers_yml: String::new(),
            namespace_search_dirs: Vec::new(),
            entity_search_dirs: Vec::new(),
            dealiaser_additionals: Vec::new(),
            bundles: Vec::new(),
            root_namespace: String::new(),
        }
    }
}

/// /!\ Mutates G (unsafe)
/// Should only be called once, in main, in the set-up phase
pub fn load_conf(args: &clap::ArgMatches<'_>) {
    /* Get config file name/path */
    let conf_file = match args.value_of("CONF_FILE") {
        Some(cf) => cf,
        None => DEFAULT_CONF_FILE,
    };
    /* Open & read file */
    let mut file = match File::open(conf_file) {
        Ok(f) => f,
        Err(e) => panic!("Could not open {} ({})", conf_file, e),
    };
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let yaml = match YamlLoader::load_from_str(&contents) {
        Ok(y) => y,
        Err(e) => panic!("Could not read {} ({})", conf_file, e),
    };

    unsafe {
        let globals_w = std::mem::transmute::<&Globals, &mut Globals>(&crate::G);
        /* Read entity_search_dirs */
        if let Err(_e) = |yaml: &Yaml| -> Result<(), std::option::NoneError> {
            let esdv = yaml["entity_search_dirs"].as_vec()?;
            for e_s_d in esdv {
                let entity_nspace_alias = e_s_d[0].as_str()?.to_owned();
                let entity_nspace = e_s_d[1].as_str()?.to_owned();
                globals_w
                    .entity_search_dirs
                    .push((entity_nspace_alias, entity_nspace));
            }
            Ok(())
        }(&yaml[0])
        {
            panic!("{}: entity_search_dirs not specified correctly", conf_file);
        }
        /* Read namespace_search_dirs */
        if let Err(_e) = |yaml: &Yaml| -> Result<(), std::option::NoneError> {
            let ns_s_dv = yaml["namespace_search_dirs"].as_vec()?;
            for ns_s_d in ns_s_dv {
                let nspace = ns_s_d[0].as_str()?.to_owned();
                let path = ns_s_d[1].as_str()?.to_owned();
                globals_w.namespace_search_dirs.push((nspace, path));
            }
            Ok(())
        }(&yaml[0])
        {
            panic!(
                "{}: namespace_search_dirs not specified correctly",
                conf_file
            );
        }
        /* Append additionnal_service_aliases dealiaser */
        if let Err(_e) = |yaml: &Yaml| -> Result<(), std::option::NoneError> {
            let a_s_a_v = yaml["additionnal_service_aliases"].as_vec()?;
            for a_s_a in a_s_a_v {
                let pointy_name = a_s_a[0].as_str()?.to_owned();
                let psr_name = a_s_a[1].as_str()?.to_owned();
                globals_w
                    .dealiaser_additionals
                    .push((psr_name, pointy_name));
            }
            Ok(())
        }(&yaml[0])
        {
            println!("{}", "additionnal_service_aliases badly formatted".red());
        }
        /* Read bundles */
        if let Err(_e) = |yaml: &Yaml| -> Result<(), std::option::NoneError> {
            let a_s_a_v = yaml["bundles"].as_vec()?;
            for a_s_a in a_s_a_v {
                let name = a_s_a["name"].as_str()?.to_owned();
                let path = a_s_a["path"].as_str()?.to_owned();
                globals_w.bundles.push((name, path));
            }
            Ok(())
        }(&yaml[0])
        {
            println!("{}", "bundles badly formatted".red());
        }
        /* Read every other global variable that's a string */
        let mut pairs = [
            (&mut globals_w.work_dir, "work_dir"),
            (&mut globals_w.project_root, "project_root"),
            (&mut globals_w.project_conf, "project_conf"),
            (&mut globals_w.project_srcs, "project_srcs"),
            (&mut globals_w.controllers_yml, "controllers_yml"),
            (&mut globals_w.root_namespace, "root_namespace"),
        ];
        for p in pairs.iter_mut() {
            if let Some(value) = yaml[0][p.1].as_str() {
                *p.0 = value.to_owned();
            }
        }
    }
}

/// /!\ Mutates G (unsafe)
/// Should only be called once, in main, in the set-up phase
pub fn load_args(args: &clap::ArgMatches<'_>) {
    unsafe {
        let globals_w = std::mem::transmute::<&Globals, &mut Globals>(&crate::G);
        let mut pairs = [
            (&mut globals_w.project_root, "PROJECT_FD"),
            (&mut globals_w.work_dir, "WORK_DIR"),
            (&mut globals_w.controllers_yml, "CONTROLLERS_CONF_YML"),
        ];
        for p in pairs.iter_mut() {
            if let Some(a) = args.value_of(p.1) {
                *p.0 = a.to_owned();
            } else if p.0.len() == 0 {
                panic!("{} needs to be set", p.1);
            }
        }
    }
}
