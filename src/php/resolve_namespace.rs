use std::path::Path;

pub fn resolve_namespace(full_name: &str) -> Option<String> {
    let project_root = crate::PROJECT_ROOT.read().unwrap();
    let name_sep_id = full_name.rfind('\\').unwrap_or(0);

    // let file_name = format!("{}.php", &full_name[name_sep_id..]);
    let partial_path = format!(
        "{}{}.php",
        &full_name[..name_sep_id].replace("\\", "/"),
        &full_name[name_sep_id..]
    );

    for namespace_root in crate::NAMESPACE_SEARCH_DIRS {
        let path = format!("{}{}{}", project_root, namespace_root, partial_path);
        // println!("resolve_namespace: testing `{}`", path);
        if Path::new(&path).exists() {
            return Some(path);
        }
    }
    return None;
}
