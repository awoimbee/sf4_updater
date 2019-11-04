use crate::dealiaser::Dealiaser;
use crate::php::transformers::FileTransformer;
use crate::php::*;

impl Php {
    // fn rm_get_transform()

    pub fn rm_get(&mut self, dealiaser: &Dealiaser) {
        println!("rm_get");
        let pile_reader = self.has_get_stack.read().unwrap();

        for class_name in pile_reader.iter() {
            println!("\tName {}", class_name);
            let classes_r = self.classes.read().unwrap();
            let class_mutex = classes_r.get(class_name).unwrap().clone();
            drop(classes_r);
            let class = class_mutex.lock().unwrap();

            if !class.has_constructor && class.parent.is_some() {

                let parent_class_name = class.parent.as_ref().unwrap();
                if self
                    .load_class(parent_class_name, Some(file_dir_path(&class.path)))
                    .is_none()
                {
                    println!("\t\tCannot load parent class `{}`", parent_class_name);
                    continue;
                }
                if self
                    .classes
                    .read()
                    .unwrap()
                    .get(parent_class_name)
                    .unwrap()
                    .lock()
                    .unwrap()
                    .has_constructor
                {
                    println!("\t\tCannot update constructors from parent & shit right now");
                    continue;
                }
            }

            let mut ft = FileTransformer::new(&class.path);

            while let Some(get_cap) = RE_GET.captures(ft.reader()) {
                let alias_match = get_cap.get(1).unwrap();

                let get_alias = alias_match.as_str();
                let alias_start = alias_match.start();
                let alias_end = alias_match.end();

                let get_namespaced = match dealiaser.dealias(get_alias) {
                    Some(nspace) => nspace,
                    None => {
                        println!("\t\tCould not dealias `{}` !", get_alias);
                        ft.reader_skip(alias_end);
                        continue;
                    }
                };

                println!("\t\t{:70} => {}", get_alias, get_namespaced);
                ft.reader_skip(alias_end);
                continue;
            }
        }
    }
}
