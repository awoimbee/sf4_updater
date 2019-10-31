use std::fs::File;
use std::io::prelude::*;

use crate::php;
use crate::php::resolve_namespace::*;
// use crate::php::*;
use crate::php::transformers;

impl php::Php {
    pub fn dealias_get_repository(&mut self) {
        println!("dealias_get_repository");
        let pile_reader = self.has_get_repository_stack.read().unwrap();

        for class_name in pile_reader.iter() {
            println!("\t{}", class_name);
            let class_mutex = self
                .classes
                .read()
                .unwrap()
                .get(class_name)
                .unwrap()
                .clone();
            let mut class = class_mutex.lock().unwrap();

            let mut contents = String::new();
            match File::open(&class.path)
                .unwrap()
                .read_to_string(&mut contents)
            {
                Err(_) => continue,
                _ => (),
            };

            while let Some(getrepo_cap) = php::RE_GETREPOSITORY_ALIAS.captures(&contents) {
                let repo_alias_cap = getrepo_cap.get(1).unwrap();
                let repo_alias = repo_alias_cap.as_str();

                let entity_full_name = match alias_to_namespace(repo_alias) {
                    Some(full_name) => full_name,
                    None => {
                        eprintln!("FUCK {}", repo_alias);
                        break;
                    }
                };
                let entity_name = php::class_name(&entity_full_name);

                let some_use_entity = class.uses.get(entity_name);
                if some_use_entity.is_none() {
                    // No use found
                    class
                        .uses
                        .insert(entity_name.to_owned(), entity_full_name.clone());
                } else if some_use_entity.unwrap() != &entity_full_name {
                    // Use for same name object but different namespace found
                    // drop(some_use);
                    println!(
                        "/!\\ Name conflict for {} ({}) in {}",
                        entity_name, entity_full_name, class.path
                    );
                    break;
                }
                let before_repo_alias = &contents[..repo_alias_cap.start() - 1];
                let new_name = format!("{}::class", entity_name);
                let after_repo_alias = &contents[repo_alias_cap.end() + 1..];

                contents = format!("{}{}{}", before_repo_alias, new_name, after_repo_alias);
            }
            contents = transformers::rewrite_uses(contents, &class);
            transformers::write_file(&contents, &class.path);
        }
    }
}
