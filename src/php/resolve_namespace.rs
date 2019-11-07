use std::path::Path;

/// full_name: Meero\Shootbundle\ClassThing
/// Returns path (../meero/src/ShootBundle/ClassThing.php)
/// search_dir is optional, it's the first directory to look for the class in
pub fn namespace_to_path(full_name: &str) -> Option<String> {
    let name_sep_id = match full_name.rfind('\\') {
        Some(i) => i + 1,
        None => 0,
    };
    let file_name = &full_name[name_sep_id..]; // yolo
    // let partial_path = &full_name[..name_sep_id].replace("\\", "/");

    for namespace_sd in &crate::G.namespace_search_dirs {
        if full_name.starts_with(&namespace_sd.0) {
            let partial_path = full_name[namespace_sd.0.len()..name_sep_id].replace("\\", "/");
            let path = format!(
                "{}{}{}{}.php",
                crate::G.project_root,
                namespace_sd.1,
                partial_path,
                file_name
            );
            if Path::new(&path).exists() {
                return Some(path);
            }
        }
    }
    None
}

/// get entity alias ('MeeroShootBundle:Location')
/// Returns namespace ('Meero\Shootbundle\Entity\Location')
pub fn entity_dealias(full_name: &str) -> Option<String> {
    let sep = full_name.rfind(':').unwrap_or(0);

    let nspace_alias = &full_name[..sep];
    let cname = &full_name[sep + 1..];

    for (g_alias, g_nspace) in &crate::G.entity_search_dirs {
        if g_alias == nspace_alias {
            return Some(format!("{}{}", g_nspace, cname));
        }
    }
    None
}
