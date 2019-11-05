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
            let mut class = class_mutex.lock().unwrap();

            if class.construct_args.len() == 0 && class.parent.is_some() {
                let parent_class_name = class.parent.as_ref().unwrap();
                if self
                    .load_class(parent_class_name, Some(file_dir_path(&class.path)))
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
                }
                else {
                    let service_short_name = match service_fname.rfind('\\') {
                        Some(i) => &service_fname[i + 1..],
                        None => &service_fname
                    };
                    let mut var_name = service_short_name.to_owned();
                    unsafe {
                        let p = (&mut var_name).as_mut_ptr();
                        let c = (*p as char).to_ascii_lowercase();
                        *p = c as u8;
                    }


                    println!("replace {} by $this->{}", full_match.as_str(), var_name);
                    if class.construct_args.len() == 0 {
                        class.uses.insert(service_short_name.to_owned(), service_fname.clone());
                        ft.reader_replace(fmatch_bounds.0, fmatch_bounds.1, &format!("$this->{}", var_name));
                        ft.new_constructor_injection(service_short_name, &var_name);
                    } else {
                        println!("FUCKING TODO");
                        ft.reader_skip(fmatch_bounds.1);
                    }

                }
                // println!("\t\t{:50} => {}: {}", get_alias, var_name, service_fname);
                // ft.reader_skip(fmatch_bounds.1);
                // continue;
                break;
            }
            ft.rewrite_uses(&class);
            ft.write_file(&class.path);
        }
    }
}
