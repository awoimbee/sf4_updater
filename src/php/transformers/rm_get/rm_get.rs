use super::dealiaser::Dealiaser;
use crate::f_find::f_find;
use crate::php::transformers::FileTransformer;
use crate::php::*;
use crate::G;
use chrono::Utc;
use colored::*;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::io::Write;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

impl Php {
    pub fn rm_get(&mut self) {
        println!("rm_get");

        let mut dealiaser = Dealiaser::new();
        let d = &mut dealiaser;
        f_find(&G.project_conf, r".*\.yml", |s| d.clone().add_from_yml(s));
        f_find(&G.project_srcs, r".*\.yml", |s| d.clone().add_from_yml(s));
        for (psr, alias) in &G.dealiaser_additionals {
            dealiaser.add(&psr, &alias);
        }
        dealiaser.checkup();

        let pile_reader = self.get_stack.read().unwrap();
        let conf_set = read_controllers_config(&crate::G.controllers_yml);
        let mut conf_to_add: Vec<&str> = Vec::new();
        for class_name in pile_reader.iter() {
            println!("\t{}: ", class_name);
            self.rm_get_in_class(class_name, &conf_set, &mut conf_to_add, &dealiaser);
        }
        if conf_to_add.len() == 0 {
            return;
        }
        let try_open = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&crate::G.controllers_yml);
        let mut yml_w_handle = match try_open {
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

    fn rm_get_in_class<'a>(
        &self,
        class_name: &'a str,
        conf_set: &BTreeSet<String>,
        conf_to_add: &mut Vec<&'a str>,
        dealiaser: &Dealiaser,
    ) {
        if !conf_set.contains(class_name) {
            conf_to_add.push(class_name);
        }
        let mut to_add_to_construt: BTreeMap<String, String> = BTreeMap::new();

        let classes_r = self.classes.read().unwrap();
        let class_mut = classes_r.get(class_name).unwrap().clone();
        drop(classes_r);

        let parent_constructor = match some_parent_has_constructor(&self, class_name) {
            Ok(v) => v,
            Err(e) => {
                println!("\t\t{}", format!("{}", e).red());
                return;
            }
        };

        let mut class = class_mut.lock().unwrap();
        println!("\t-> {}:", class.path);

        if class.children.len() > 0 {
            println!("\t\tCannot update children as of now");
            return;
        }
        // if class.construct_args.len() == 0 && parent_constructor {
        //     // todo: if name ends w/ Command, fuck it and just add parent::__construct(null)
        //     // if class_name.ends_with(pat: P)
        //     println!(
        //         "\t\t{}",
        //         "Cannot create constructor that constructs parent right now".red()
        //     );
        //     return;
        // }

        let mut ft = match FileTransformer::new(&class.path) {
            Some(ft) => ft,
            None => return,
        };

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

            if let Some(arg) = class.construct_arg_type(&service_fname) {
                println!(
                    "\t\t{} {}",
                    "Service already injected, code path not written".red(),
                    format!("{} -> $this->{}", get_alias, arg.name)
                );
                ft.reader_skip(fmatch_bounds.1);
                continue;
            } else if class.children.len() > 0 {
                println!("\t\t{}", "Class has children, code path not written".red());
                ft.reader_skip(fmatch_bounds.1);
                continue;
            } else {
                let mut service_short_name = match service_fname.rfind('\\') {
                    Some(i) => &service_fname[i + 1..],
                    None => &service_fname,
                };
                if service_short_name.ends_with("Interface") {
                    service_short_name =
                        &service_short_name[..service_short_name.rfind("Interface").unwrap()];
                }
                let mut var_name = service_short_name.to_owned();
                var_name.get_mut(0..1).unwrap().make_ascii_lowercase();

                println!(
                    "{}",
                    format!("\t\treplace {} by $this->{}", full_match.as_str(), var_name).green()
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
            }
        }
        ft.rewrite_uses(&class);
        ft.add_to_constructor(&to_add_to_construt);
        ft.write_file(&class.path);
    }
}

fn read_controllers_config(file_path: &str) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    let mut ft = match FileTransformer::new(file_path) {
        Some(ft) => ft,
        None => return set,
    };

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
    set
}

/// LOCKS MUTEX ON CLASS
fn some_parent_has_constructor(php: &Php, class: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let classes_r = php.classes.read().unwrap();
    let mut class = classes_r.get(class).unwrap().lock().unwrap();
    while let Some(parent_name) = class.parent.as_ref() {
        php.load_class(&parent_name)?;
        class = classes_r.get(parent_name.as_str()).unwrap().lock().unwrap();
        if class.construct_args.len() > 0 {
            return Ok(true);
        }
    }
    return Ok(false);
}
