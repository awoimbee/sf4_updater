use crate::dealiaser::Dealiaser;
use crate::php::transformers::FileTransformer;
use crate::php::*;
use chrono::Utc;
use colored::*;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::io::Write;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

fn read_controllers_config(file_path: &str) -> BTreeSet<String> {
    let mut ft = FileTransformer::new(file_path);
    let mut set = BTreeSet::new();

    let yaml = match YamlLoader::load_from_str(ft.get_mut()) {
        Ok(y) => y,
        Err(e) => {
            println!("/!\\invalid Yaml in `{}`: {}", file_path, e);
            return set;
        }
    };

    if yaml.len() == 0 || yaml[0]["services"] == Yaml::BadValue {
        return set;
    }

    if yaml[0]["services"]["_defaults"] == Yaml::BadValue
        || yaml[0]["services"]["_defaults"]["autowire"] == Yaml::BadValue
        || yaml[0]["services"]["_defaults"]["autowire"]
            .as_bool()
            .unwrap_or(false)
            != true
        || yaml[0]["services"]["_defaults"]["public"] == Yaml::BadValue
        || yaml[0]["services"]["_defaults"]["public"]
            .as_bool()
            .unwrap_or(false)
            != true
    {
        let yml_header = concat!(
            " services:\n",
            "   _defaults:\n",
            "     autowire: true\n",
            "     public: true\n"
        );
        eprintln!(
            "{}",
            format!("{} NEEDS to contain:\n{}", file_path, yml_header).red()
        );
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
    pub fn rm_get(&mut self, dealiaser: &Dealiaser) {
        println!("rm_get");
        let pile_reader = self.has_get_stack.read().unwrap();

        let conf_set = read_controllers_config(&crate::G.controllers_yml);
        let mut conf_to_add: Vec<&str> = Vec::new();

        for class_name in pile_reader.iter() {
            println!("\t{}:", class_name);

            if !conf_set.contains(class_name) {
                conf_to_add.push(class_name);
            }
            let mut to_add_to_construt: BTreeMap<String, String> = BTreeMap::new();

            let classes_r = self.classes.read().unwrap();
            let class_mutex = classes_r.get(class_name).unwrap().clone();
            drop(classes_r);
            let mut class = class_mutex.lock().unwrap();

            if class.construct_args.len() == 0 && class.parent.is_some() {
                let parent_class_name = class.parent.as_ref().unwrap();
                if self.load_class(parent_class_name).is_none() {
                    println!("\t\t{} `{}`", "Cannot load class".red(), parent_class_name);
                    continue;
                }
                let classes_r = self.classes.read().unwrap();
                let parent = classes_r.get(parent_class_name).unwrap();
                if parent.lock().unwrap().construct_args.len() > 0 {
                    println!(
                        "\t\t{}",
                        "Cannot create constructor that constructs parent right now".red()
                    );
                    continue;
                }
            }

            let mut ft = FileTransformer::new(&class.path);

            while let Some(get_cap) = RE_GET.captures(ft.reader()) {
                let full_match = get_cap.get(0).unwrap();
                let alias_match = get_cap.get(1).unwrap();
                let fmatch_bounds = (full_match.start(), full_match.end());

                let get_alias = alias_match.as_str();
                let service_fname = match dealiaser.dealias(get_alias) {
                    Some(nspace) => nspace,
                    None => {
                        println!("\t\t{} `{}` !", "Could not dealias".yellow(), get_alias);
                        ft.reader_skip(fmatch_bounds.1);
                        continue;
                    }
                };

                if let Some(_arg) = class.construct_arg_type(&service_fname) {
                    println!(
                        "\t\t{}",
                        "Service already injected, code path not written".red()
                    );
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

                    if class.construct_args.len() == 0 {
                        println!(
                            "{}",
                            format!("\t\treplace {} by $this->{}", full_match.as_str(), var_name)
                                .green()
                        );
                        class
                            .uses
                            .insert(service_short_name.to_owned(), service_fname.clone());
                        ft.reader_replace(
                            fmatch_bounds.0,
                            fmatch_bounds.1,
                            &format!("$this->{}", var_name),
                        );
                        to_add_to_construt.insert(service_short_name.to_owned(), var_name);
                    } else {
                        println!(
                            "\t\t{}",
                            "Need to update existing constructor, code path not written".red()
                        );
                        ft.reader_skip(fmatch_bounds.1);
                    }
                }
            }
            ft.rewrite_uses(&class);
            ft.add_to_constructor(&to_add_to_construt);
            ft.write_file(&class.path);
        }
        if conf_to_add.len() > 0 {
            let mut yml_w_handle = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&crate::G.controllers_yml)
            {
                Ok(f) => f,
                Err(e) => {
                    println!("\t\tCould not open controllers conf ({})", e);
                    return;
                }
            };
            let new_yml = format!(
                "\n# Auto-generated at {}\n{}",
                Utc::now(),
                conf_to_add
                    .iter()
                    .map(|s| format!("  {}: ~\n", s))
                    .collect::<String>()
            );
            yml_w_handle.write(new_yml.as_bytes()).unwrap();
        }
    }
}
