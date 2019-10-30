use regex::Regex;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;

use crate::dealiaser::Dealiaser;
use crate::php::resolve_namespace::*;
use crate::php::*;

fn add_use(class_content: &str, use_text: &str) -> String {
	let where_ = RE_ALL_USE.find(class_content).unwrap().end();
	let file_begin = &class_content[..where_];
	let file_end = &class_content[where_ + 1..];
	let middle = format!("use {};\n", use_text);
	return format!("{}{}{}", file_begin, middle, file_end);
}


impl Php {
    pub fn rm_get_repository(&mut self) {
        let pile_reader = self.work_stack.read().unwrap();

        for class_name in pile_reader.iter() {
            println!("Name {}", class_name);

            let classes_r = self.classes.read().unwrap();
            let class = classes_r.get(class_name).unwrap().clone();
            drop(classes_r);

			let mut contents = String::new();
			File::open(&class.path).unwrap().read_to_string(&mut contents).unwrap_or(0);

			while let Some(cap) = RE_GETREPOSITORY.captures(&contents) {
				println!("capture: {:?}", cap);
				let repo_alias_cap = cap.get(1).unwrap();
				let repo_alias = &cap[1];
				let repo_namespace = resolve_entity_namespace(repo_alias).unwrap_or(format!(""));
				let repo_name = &repo_namespace[repo_namespace.rfind('\\').unwrap_or(0) + 1..];

				let where_use = RE_ALL_USE.find(&contents).unwrap().end();


				let before_use = &contents[..where_use];
				let use_ = format!("\nuse {};\n", repo_namespace);
				let after_use = &contents[where_use + 1..repo_alias_cap.start()-1];
				let new_name = format!("class:{}", repo_name);
				let after_repo_alias = &contents[repo_alias_cap.end()+1..];

				println!("Alias {} => {}", repo_alias, new_name);

				contents = format!(
					"{}{}{}{}{}",
					before_use,
					use_,
					after_use,
					new_name,
					after_repo_alias
				);
				// contents = format!("PUTENEGRE");

				println!("#############################\n\n\n{}\n\n\n#########################", contents);
			}
			let mut file_w = OpenOptions::new()
				.write(true)
				.truncate(true)
				.open(&class.path)
				.unwrap();
			file_w.write(&contents.as_bytes()).unwrap();
            // let get_cap = RE_GET.captures_iter(&contents);

            // for get in get_cap {
            //     let get_alias = &get[1];
            //     let get_namespaced = match dealiaser.dealias(get_alias) {
            //         Some(nspace) => nspace,
            //         None => continue // println!("Could not dealias {} !", get_alias);
            //     };
            //     println!("\t{:50} => {}", get_alias, get_namespaced);
            //     // let new_contents = add_use(&contents, class.idx_use_end, &get_namespaced);
            //     // for cap_use in RE_USE.captures_iter(&contents) {
            // }
        }
    }
}
