use crate::dealiaser::Dealiaser;
use crate::php::transformers::FileTransformer;
use crate::php::*;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::Write;
use std::io::prelude::*;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;
use std::time::SystemTime;
use chrono::{DateTime, Utc};

fn read_controllers_config(file_path: &str) -> BTreeSet<String> {
    println!("read_controllers_config: {}", file_path);
    let mut ft = FileTransformer::new(file_path);
    let mut set = BTreeSet::new();

    let yaml = match YamlLoader::load_from_str(ft.get_mut()) {
        Ok(y) => y,
        Err(e) => {
            eprintln!("/!\\invalid Yaml in `{}`: {}", file_path, e);
            return set;
        }
    };
    if yaml[0]["_defaults"] == Yaml::BadValue
    || yaml[0]["_defaults"]["autowire"] == Yaml::BadValue
    || yaml[0]["_defaults"]["autowire"].as_bool().unwrap_or(false) != true
    || yaml[0]["_defaults"]["public"] == Yaml::BadValue
    || yaml[0]["_defaults"]["public"].as_bool().unwrap_or(false) != true {
        let yml_header = concat!(
            " services:\n",
            "   _defaults:\n",
            "     autowire: true\n",
            "     public: true\n"
        );
        eprintln!("/!\\ {} NEEDS to contain the following:\n{}\n", file_path, yml_header);
    }


    let services = match &yaml[0]["services"] {
        Yaml::BadValue => return set,
        s => s.as_hash().unwrap(),
    };
    for (s_name, _s_opts) in services {
        set.insert(s_name.as_str().unwrap().to_owned());
    }
    return set;
}



impl Php {
    // fn rm_get_transform()

    pub fn rm_get(&mut self, dealiaser: &Dealiaser) {
        println!("rm_get");
        let pile_reader = self.has_get_stack.read().unwrap();

        let c_r = crate::CONTROLLERS_YML.read().unwrap();
        let conf_set = read_controllers_config(c_r.as_ref());
        let mut conf_to_add: Vec<&str> = Vec::new();

        for class_name in pile_reader.iter() {
            println!("\tName {}", class_name);
            if !conf_set.contains(class_name) {
                conf_to_add.push(class_name);
            }
            // vec of (typeName, varName)
            // let mut to_add_to_construt: Vec<(String, String)> = Vec::new();

            // BTreeMap<VarType, VarName>
            let mut to_add_to_construt: BTreeMap<String, String> = BTreeMap::new();

            let classes_r = self.classes.read().unwrap();
            let class_mutex = classes_r.get(class_name).unwrap().clone();
            drop(classes_r);
            let mut class = class_mutex.lock().unwrap();

            if class.construct_args.len() == 0 && class.parent.is_some() {
                let parent_class_name = class.parent.as_ref().unwrap();
                if self
                    .load_class(parent_class_name)
                    .is_none()
                {
                    println!("\t\tCannot load parent class `{}`", parent_class_name);
                    continue;
                }
                let classes_r = self.classes.read().unwrap();
                let parent = classes_r.get(parent_class_name).unwrap();
                if parent.lock().unwrap().construct_args.len() > 0 {
                    println!("\t\tCannot update constructors from parent & shit right now");
                    continue;
                }
            }

            let mut ft = FileTransformer::new(&class.path);

            while let Some(get_cap) = RE_GET.captures(ft.reader()) {
                let full_match = get_cap.get(0).unwrap();
                let alias_match = get_cap.get(1).unwrap();

                let fmatch_bounds = (full_match.start(), full_match.end());
                let amatch_bounds = (alias_match.start(), alias_match.end());

                let get_alias = alias_match.as_str();
                let service_fname = match dealiaser.dealias(get_alias) {
                    Some(nspace) => nspace,
                    None => {
                        println!("\t\tCould not dealias `{}` !", get_alias);
                        ft.reader_skip(fmatch_bounds.1);
                        continue;
                    }
                };

                if let Some(arg) = class.construct_arg_type(&service_fname) {
                    println!("SERVICE IS ALREADY INJECTED, TODO");
                    ft.reader_skip(fmatch_bounds.1);
                    continue;
                } else {
                    let service_short_name = match service_fname.rfind('\\') {
                        Some(i) => &service_fname[i + 1..],
                        None => &service_fname,
                    };
                    let mut var_name = service_short_name.to_owned();
                    unsafe {
                        let p = (&mut var_name).as_mut_ptr();
                        let c = (*p as char).to_ascii_lowercase();
                        *p = c as u8;
                    }

                    println!("replace {} by $this->{}", full_match.as_str(), var_name);
                    if class.construct_args.len() == 0 {
                        class.uses.insert(
                            service_short_name.to_owned(),
                            service_fname.clone()
                        );
                        // println!("Class uses add: {:30} -> {}", service_short_name, service_fname);
                        ft.reader_replace(
                            fmatch_bounds.0,
                            fmatch_bounds.1,
                            &format!("$this->{}", var_name),
                        );
                        to_add_to_construt.insert(service_short_name.to_owned(), var_name);
                        // to_add_to_construt.push((service_short_name.to_owned(), var_name));
                    } else {
                        println!("FUCKING TODO");
                        ft.reader_skip(fmatch_bounds.1);
                    }
                }
                // println!("\t\t{:50} => {}: {}", get_alias, var_name, service_fname);
                // ft.reader_skip(fmatch_bounds.1);
                // continue;
                // break;
            }
            ft.rewrite_uses(&class);
            ft.add_to_constructor(&to_add_to_construt);
            ft.write_file(&class.path);
        }
        if conf_to_add.len() > 0 {
            let mut yml_w_handle = match OpenOptions::new().create(true).append(true).open(c_r.as_ref() as &str) {
                Ok(f) => f,
                Err(e) => {
                    println!("\nCould not open controllers conf ({})", e);
                    return;
                }
            };
            let new_yml = format!(
                "\n# Auto-generated at {}\n{}",
                Utc::now(),
                conf_to_add.iter().map(|s| format!("  {}: ~\n", s)).collect::<String>()
            );
            yml_w_handle.write(new_yml.as_bytes()).unwrap();
        }
    }
}
