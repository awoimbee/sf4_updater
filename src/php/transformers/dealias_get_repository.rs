use crate::php;
use crate::php::resolve_namespace::*;
use crate::php::transformers::FileTransformer;
use colored::*;

impl php::Php {
    pub fn dealias_get_repository(&mut self) {
        println!("dealias_get_repository");
        let pile_reader = self.get_repository_stack.read().unwrap();

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
            let mut ft = FileTransformer::new(&class.path);

            while let Some(getrepo_cap) = php::RE_GETREPOSITORY_ALIAS.captures(ft.reader()) {
                let repo_alias_cap = getrepo_cap.get(1).unwrap();
                let repo_alias = repo_alias_cap.as_str();

                let entity_full_name = match entity_dealias(repo_alias) {
                    Some(full_name) => full_name,
                    None => {
                        eprintln!(
                            "{}",
                            format!("\t\tCould not dealias entity: {}", repo_alias).red()
                        );
                        let end = repo_alias_cap.end().clone();
                        ft.reader_skip(end);
                        continue;
                    }
                };
                let entity_name = php::class_name(&entity_full_name);

                match class.uses.get(entity_name) {
                    Some(use_entity) => {
                        if use_entity != &entity_full_name {
                            println!(
                                "{}",
                                format!(
                                    "\t\tName conflict for {} ({}) in {}",
                                    entity_name, entity_full_name, class.path
                                )
                                .red()
                            );
                            let end = repo_alias_cap.end();
                            ft.reader_skip(end);
                            continue;
                        }
                    }
                    None => {
                        class
                            .uses
                            .insert(entity_name.to_owned(), entity_full_name.clone());
                    }
                };
                let new = format!("{}::class", entity_name);
                let start = repo_alias_cap.start();
                let end = repo_alias_cap.end();
                ft.reader_replace(start - 1, end + 1, &new);
            }
            ft.rewrite_uses(&class);
            ft.write_file(&class.path);
        }
    }
}
