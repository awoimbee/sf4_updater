use std::path::Path;

/// full_name: Meero\Shootbundle\ClassThing
/// Returns path (../meero/src/ShootBundle/ClassThing.php)
/// search_dir is optional, it's the first directory to look for the class in
pub fn namespace_to_path(full_name: &str, search_dir: Option<&str>) -> Option<String> {
    let project_root = crate::PROJECT_ROOT.read().unwrap();
    let name_sep_id = full_name.rfind('\\').unwrap_or(0);
    let file_name = &full_name[name_sep_id+1..]; // yolo
    let partial_path = &full_name[..name_sep_id+1].replace("\\", "/");

    if search_dir.is_some() {
        let search_dir = search_dir.unwrap(); //project_root
        let path = format!("{}{}.php", search_dir, file_name);

        println!("SEARCH_DIR: {}", path);
        if Path::new(&path).exists() {
            return Some(path);
        }

    }

    let partial_path = format!("{}{}.php", file_name, partial_path);

    for namespace_root in crate::NAMESPACE_SEARCH_DIRS {
        let path = format!("{}{}{}", project_root, namespace_root, partial_path);
        println!("resolve_namespace: testing `{}`", path);
        if Path::new(&path).exists() {
            return Some(path);
        }
    }
    return None;
}

/// get entity alias ('MeeroShootBundle:Location')
/// Return namespace ('Meero\Shootbundle\Entity\Location')
pub fn alias_to_namespace(full_name: &str) -> Option<String> {
    let sep = full_name.rfind(':').unwrap_or(0);

    let nspace_alias = &full_name[..sep];
    let cname = &full_name[sep + 1..];

    for (g_alias, g_nspace) in crate::ENTITY_SEARCH_DIRS {
        if *g_alias == nspace_alias {
            return Some(format!("{}{}", g_nspace, cname));
        }
    }
    return None;
}
