use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

mod class_extractor;
pub mod resolve_namespace;
pub mod transformers;

const RSTR_CLASS: &str =
    r"\n(?:abstract )?class (?P<name>[^ \n]*)(?: extends (?P<parent>[^ \n]*))?";
const RSTR_NAMESPACE: &str = r"\nnamespace ([^ ;]*);\n"; // .get(1): namespace;
const RSTR_ALL_USE: &str = r"(?:\nuse[^;]*;)+"; // CAP WHOLE GROUP
const RSTR_USE: &str = r"\nuse (?P<class>[^ ;]*)(?: as (?P<alias>[^ ;]*))?;";
const RSTR_CONSTRUCT: &str =
    r"(?:/\*\*(?:[*][^/]|[^*])*\*/\s*)?public function __construct\((?P<args>[^)]*)\)[^{]*\{";
const RSTR_METH_N_DOC: &str =
    //            doc-block            |       |         visibility         |     |        | name |     args
    r"(?:/\*\*(?:[*][^/]|[^*])*\*/\s*)?\n[ \t]*(?:public|private|protected)?[ \t]*function [^ (]*";
const RSTR_GET: &str = r"(?:(?:\$this)|(?:\$container)|(?:\$[^-$ ]*->getContainer\(\))|(?:\$[^-$ ]*->container))->get\('(?P<alias>.*?)'\)";

//  r"\$this->get\('(.*?)'\)"; // .get(0): $this->get; .get(1): class
/// Only finds the getrepository that uses the 'alias' name
/// // .get(0): $this->get; .get(1): class
const RSTR_GETREPOSITORY_ALIAS: &str = r"->getRepository\('(.*?:.*?)'\)";

lazy_static! {
    static ref RE_CLASS: Regex = Regex::new(RSTR_CLASS).unwrap();
    static ref RE_NAMESPACE: Regex = Regex::new(RSTR_NAMESPACE).unwrap();
    static ref RE_ALL_USE: Regex = Regex::new(RSTR_ALL_USE).unwrap();
    static ref RE_USE: Regex = Regex::new(RSTR_USE).unwrap();
    static ref RE_CONSTRUCT: Regex = Regex::new(RSTR_CONSTRUCT).unwrap();
    static ref RE_METH_N_DOC: Regex = Regex::new(RSTR_METH_N_DOC).unwrap();
    static ref RE_GET: Regex = Regex::new(RSTR_GET).unwrap();
    static ref RE_GETREPOSITORY_ALIAS: Regex = Regex::new(RSTR_GETREPOSITORY_ALIAS).unwrap();
}

#[derive(Debug)]
/// Exemple:
/// ```php
/// (..)function fn(typeh $name = def_val) {
///     $this->in_class_name = $name;
/// }
/// ```
struct Arg {
    name: String,
    typeh: Option<String>,
    def_val: Option<String>,
    in_class_name: Option<String>,
}

#[derive(Debug)]
struct Class {
    path: String,
    children: Vec<String>,
    parent: Option<String>,
    uses: HashMap<String, String>, // Alias -- ClassFullName
    construct_args: Vec<Arg>,      // if 0 -> no constructor
    has_get: bool,
    has_get_repository: bool,
}

#[derive(Debug, Clone)]
pub struct Php {
    classes: Arc<RwLock<HashMap<String, Arc<Mutex<Class>>>>>,
    has_get_stack: Arc<RwLock<Vec<String>>>,
    has_get_repository_stack: Arc<RwLock<Vec<String>>>,
}

impl Class {
    pub fn new() -> Class {
        Class {
            path: String::new(),
            children: Vec::new(),
            parent: None,
            uses: HashMap::new(),
            construct_args: Vec::new(),
            has_get: false,
            has_get_repository: false,
        }
    }
    // pub fn construct_arg_named(&self, name: &str) -> Option<&Arg> {
    //     for ca in self.construct_args.iter() {
    //         if ca.name == name {
    //             return Some(ca);
    //         }
    //     }
    //     return None;
    // }
    // pub fn construct_arg_named_mut(&mut self, name: &str) -> Option<&mut Arg> {
    //     for ca in self.construct_args.iter_mut() {
    //         if ca.name == name {
    //             return Some(ca);
    //         }
    //     }
    //     return None;
    // }
    pub fn construct_arg_type(&self, typeh: &str) -> Option<&Arg> {
        for ca in self.construct_args.iter() {
            if ca.typeh.is_some() && ca.typeh.as_ref().unwrap() == typeh {
                return Some(ca);
            }
        }
        None
    }
    // pub fn construct_arg_type_mut(&mut self, typeh: &str) -> Option<&mut Arg> {
    //     for ca in self.construct_args.iter_mut() {
    //         if ca.typeh.is_some() && ca.typeh.as_ref().unwrap() == typeh {
    //             return Some(ca);
    //         }
    //     }
    //     return None;
    // }
}

impl Php {
    pub fn new() -> Php {
        let classes = Arc::new(RwLock::new(HashMap::new()));
        let has_get_stack = Arc::new(RwLock::new(Vec::new()));
        let has_get_repository_stack = Arc::new(RwLock::new(Vec::new()));
        Php {
            classes,
            has_get_stack,
            has_get_repository_stack,
        }
    }

    pub fn load_class(&self, class_full_name: &str) -> Option<()> {
        if self.classes.read().unwrap().get(class_full_name).is_some() {
            return Some(());
        }

        if let Some(class_path) = resolve_namespace::namespace_to_path(class_full_name) {
            // println!("add_from_php: {} {}", class_full_name, class_path);
            self.add_from_php(&class_path);
            if let Some(_) = self.classes.read().unwrap().get(class_full_name) {
                return Some(());
            }
            // println!("BIG FRICK: {} from {}", class_full_name, class_path);
        }
        // println!("Resolve namespace failed for {} ??", class_full_name);
        None
    }
}

/// Returns slice from the class full name
///
/// Ex: `Root\MyBundle\Thing\Service` -> `Service`
fn class_name(class_full_name: &str) -> &str {
    match class_full_name.rfind('\\') {
        Some(i) => &class_full_name[i + 1..],
        None => class_full_name,
    }
}

// /// Returns slice from the class full name
// ///
// /// Ex: `Root\MyBundle\Thing\Service` -> `Root\MyBundle\Thing\`
// ///
// /// Ex: `Test` -> `Test`
// fn class_namespace(class_full_name: &str) -> &str {
//     match class_full_name.rfind('\\') {
//         Some(i) => &class_full_name[..i],
//         None => class_full_name,
//     }
// }

// /// Returns slice from path
// /// ## Exemple
// /// ```
// /// let class_path = "Root/src/MyBundle/Service.php"
// /// assert_eq!(file_dir_path(class_path, "Root/src/MyBundle/"));
// ///
// /// ```
// fn file_dir_path(class_path: &str) -> &str {
//     assert_eq!("a", "a");
//     match class_path.rfind('/') {
//         Some(i) => &class_path[..i],
//         None => class_path,
//     }
// }
