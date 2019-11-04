use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

mod class_extractor;
mod method_extractor;
mod php_parser;
pub mod resolve_namespace;
pub mod transformers;

lazy_static! {
    static ref RE_CLASS: Regex = // .get(2): class; .get(4): parent;
        Regex::new(r"\n(abstract )?class ([^ \n]*)( extends ([^ \n]*))?").unwrap();
    static ref RE_NAMESPACE: Regex = // .get(1): namespace;
        Regex::new(r"\nnamespace ([^ ;]*);\n").unwrap();
    /// Captures the whole use group, don't use global matching on this one
    static ref RE_ALL_USE: Regex =
        Regex::new(r"(?:\nuse[^;]*;)+").unwrap();
    static ref RE_USE: Regex = // .get(1): real_class_name;  .get(3): as_alias;
        Regex::new(r"\nuse ([^ ;]*)( as ([^ ;]*))?;").unwrap();
    static ref RE_CONSTRUCT: Regex = // .get(0): 'pub... {'; .get(1): (*args*)
        Regex::new(r"public function __construct\(([^)]*)\)[^{]*\{").unwrap();
    static ref RE_NO_CONSTRUCT: Regex =
        Regex::new(r"(?-s)\n[ \t]*.*?function [^ (]*").unwrap();
    static ref RE_GET: Regex = // .get(0): $this->get; .get(1): class
        Regex::new(r"\$this->get\('(.*?)'\)").unwrap();
    /// Only finds the getrepository that uses the 'alias' name
    static ref RE_GETREPOSITORY: Regex = // .get(0): $this->get; .get(1): class
        Regex::new(r"->getRepository\('(.*?)'\)").unwrap();
    static ref RE_GETREPOSITORY_ALIAS: Regex = // .get(0): $this->get; .get(1): class
        Regex::new(r"->getRepository\('(.*?:.*?)'\)").unwrap();

}

#[derive(Debug)]
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
    pub fn construct_arg_named(&self, name: &str) -> Option<&Arg> {
        for ca in self.construct_args.iter() {
            if ca.name == name {
                return Some(ca);
            }
        }
        return None;
    }
    pub fn construct_arg_named_mut(&mut self, name: &str) -> Option<&mut Arg> {
        for ca in self.construct_args.iter_mut() {
            if ca.name == name {
                return Some(ca);
            }
        }
        return None;
    }
    pub fn construct_arg_type(&self, typeh: &str) -> Option<&Arg> {
        for ca in self.construct_args.iter() {
            if ca.typeh.is_some() && ca.typeh.as_ref().unwrap() == typeh {
                return Some(ca);
            }
        }
        return None;
    }
    pub fn construct_arg_type_mut(&mut self, typeh: &str) -> Option<&mut Arg> {
        for ca in self.construct_args.iter_mut() {
            if ca.typeh.is_some() && ca.typeh.as_ref().unwrap() == typeh {
                return Some(ca);
            }
        }
        return None;
    }
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

    pub fn load_class(&self, class_full_name: &str, path: Option<&str>) -> Option<()> {
        let classes_r = self.classes.read().unwrap(); //_reader_factory.handle();
                                                      // let class = classes_r.get(class_full_name);

        if classes_r.get(class_full_name).is_none() {
            drop(classes_r);
            if let Some(class_path) = resolve_namespace::namespace_to_path(class_full_name) {
                self.add_from_php(&class_path);
                if let Some(_parent) = self.classes.read().unwrap().get(class_full_name) {
                    return Some(());
                }
            }
            return None;
        }
        return Some(());
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

/// Returns slice from the class full name
///
/// Ex: `Root\MyBundle\Thing\Service` -> `Root\MyBundle\Thing\`
///
/// Ex: `Test` -> `Test`
fn class_namespace(class_full_name: &str) -> &str {
    match class_full_name.rfind('\\') {
        Some(i) => &class_full_name[..i],
        None => class_full_name,
    }
}

/// Returns slice from path
/// ## Exemple
/// ```
/// let class_path = "Root/src/MyBundle/Service.php"
/// assert_eq!(file_dir_path(class_path, "Root/src/MyBundle/"));
///
/// ```
fn file_dir_path(class_path: &str) -> &str {
    assert_eq!("a", "a");
    match class_path.rfind('/') {
        Some(i) => &class_path[..i],
        None => class_path,
    }
}
