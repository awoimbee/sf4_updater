use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

mod class_extractor;
mod php_parser;
pub mod resolve_namespace;
pub mod transformers;

// public function __construct\(([^)]*)\)[^{]*{

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
        Regex::new(r"\n[ \t]*.*?function [^ (]*").unwrap();
    static ref RE_GET: Regex = // .get(0): $this->get; .get(1): class
        Regex::new(r"\$this->get\('(.*?)'\)").unwrap();
    /// Only finds the getrepository that uses the 'alias' name
    static ref RE_GETREPOSITORY: Regex = // .get(0): $this->get; .get(1): class
        Regex::new(r"->getRepository\('(.*?)'\)").unwrap();
    static ref RE_GETREPOSITORY_ALIAS: Regex = // .get(0): $this->get; .get(1): class
        Regex::new(r"->getRepository\('(.*?:.*?)'\)").unwrap();

}

#[derive(Debug)]
struct Class {
    path: String,
    children: Vec<String>,
    parent: Option<String>,
    uses: HashMap<String, String>, // Name \ Class
    // idx_use_end: usize,
    // idx_construct_start: usize,
    has_constructor: bool,
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
        let children = Vec::new();
        let parent = None;
        let uses = HashMap::new();
        let path = String::new();
        Class {
            path,
            children,
            parent,
            uses,
            // idx_use_end: 0,
            // idx_construct_start: 0,
            has_constructor: false,
            has_get: false,
            has_get_repository: false,
        }
    }
}

// impl Clone for Class {
//     #[inline]
//     fn clone(&self) -> Self {
//         Class {
//             path: self.path.clone(),
//             children: self.children.clone(),
//             parent: self.parent.clone(),
//             uses: self.uses.clone(),
//             // idx_use_end: self.idx_use_end,
//             // idx_construct_start: self.idx_construct_start,
//             has_constructor: self.has_constructor,
//             has_get: self.has_get,
//             has_get_repository: self.has_get
//         }
//     }
// }

// impl evmap::ShallowCopy for Class {
//     #[inline]
//     unsafe fn shallow_copy(&mut self) -> Self {
//         self.clone()
//     }
// }

// impl PartialEq for Class {
//     fn eq(&self, other: &Self) -> bool {
//         self.path == other.path
//     }
// }
// impl Eq for Class {}

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
        let classes_r = self.classes.read().unwrap(); //_reader_factory.handle();

        let class = classes_r.get(class_full_name);
        if class.is_none() {
            if let Some(class_path) = resolve_namespace::resolve_namespace(class_full_name) {
                self.add_from_php(&class_path);
                if let Some(_parent) = classes_r.get(class_full_name) {
                    return Some(());
                }
            }
            return None;
        }
        return Some(());
    }
}

/// Returns slice of the class full name
/// Ex: `Root\MyBundle\Thing\Service` -> `Service`
fn class_name(class_full_name: &str) -> &str {
    match class_full_name.rfind('\\') {
        Some(i) => &class_full_name[i + 1..],
        None => class_full_name,
    }
}
