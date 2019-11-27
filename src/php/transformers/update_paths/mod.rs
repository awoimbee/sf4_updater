mod dealias_path;
mod file_mover;
mod update_controller_path;
mod update_view_path;

use super::super::Php;
use super::FileTransformer;
use crate::f_find::f_find;
use crate::G;
use dealias_path::dealias_path;
use file_mover::FileMover;
use file_mover::MoveWhat;
use regex::Regex;
use update_controller_path::update_controller_path;
use update_view_path::update_view_path;

/// Colon path:
///     RootMyBundle:path/inside/Resources/views:filename.type
///     RootMyBundle:path/inside/Contoller/file:action
/// Short bundle path:
///     @RootMy/path/inside/Resources/views/filename.type
/// Bundle path:
///     @RootMyBundle/path/filename.type
/// std path:
///     %kernel.project_dir%/src/path/filename.type
///     ../../../filename.type
///     ./(IN_PROOT-IN_CUR_DIR)/path/filename.type
///     src/path/filename.type
///     src/path/filename
/// meeroshoot / meeroshowcase -> src/*Bundle/Resources/
/// /bundles/meeroshoot <- s3 assets ?

impl Php {
    pub fn update_paths(self, what: u32) {
        println!("update_paths");
        let mut fm: FileMover = FileMover::new();

        fm.which_files = unsafe { std::mem::transmute(what) };
        f_find(&G.project_root, ".*", |s| foreach_file(s, &mut fm));
        fm.git_mv();
    }
}

fn foreach_file(file: &str, fm: &mut FileMover) {
    lazy_static! {
        static ref RE_PATH: Regex = build_path_regex();
    }

    let mut ft = match FileTransformer::new(&file) {
        Some(ft) => ft,
        None => return,
    };

    while let Some(path_cap) = RE_PATH.captures(ft.reader()) {
        let start = path_cap.get(0).unwrap().start();
        let end = path_cap.get(0).unwrap().end();
        let path = match dealias_path(&path_cap, file) {
            Ok(p) => p,
            Err(e) => {
                println!("\tCould not transform path ({}): {}", e, &path_cap[0]);
                ft.reader_skip(end);
                continue;
            }
        };
        // println!("{:80} -> {}", &path_cap[0], path);

        // path to controller -> namespace (`controller: App\Controller\BlogController::list`)
        // path to twig -> relative file path from `templates/` (app/Resources/views at first)
        // path to mjml -> relative path to project_dir ? (gilpfile.js: gulp.task('mjml', function() {)

        match update_path(&path, file, fm) {
            Ok(path) => ft.reader_replace(start, end, &path),
            Err(e) => {
                println!("{}\t({:70})\tcontext: {}", e, &path_cap[0], file);
                ft.reader_skip(end);
                continue;
            }
        };
    }
    ft.write_file(file);
}

fn update_path(path: &str, context: &str, fm: &mut FileMover) -> Result<String, &'static str> {
    if fm.which_files.contains(MoveWhat::CONTROLLERS)
    && path.contains("/Controller/") {
        return update_controller_path(&path, context);
    }
    if fm.which_files.contains(MoveWhat::TEMPLATES)
    && path.contains("Resources/views") {
        return update_view_path(&path, fm);
    }

    // println!("path not recognised: {}", path);
    return Err("Path not recognised");
    // Ok(path.to_owned())
}

fn build_path_regex() -> Regex {
    let bundles_reg_names = G.bundles.iter().map(|(n, _)| format!("{}|", n));
    let mut bundles_reg_str = bundles_reg_names.collect::<String>();
    bundles_reg_str.pop();
    let reg_str = format!(
        concat!(
            r#"(?P<colon>{root_nspace}(?P<colon_bundle>{bundles})Bundle:(?P<colon_path>[^: ]*):(?P<colon_file>[^()"'\s ,{{}}]*))"#,
            r#"|(?P<shortBundle>@{root_nspace}(?P<shortBundle_bundle>{bundles})Bundle/(?P<shortBundle_path>[^()"'\s, {{}}]*))"#,
            r#"|(?P<short>@?{root_nspace}(?P<short_bundle>{bundles})/(?P<short_path>[^()"'\s,{{}}]*))"#,
            // r#"|(?P<std>['"][^ \n\t\v\r:;`]*?(?P<std_bundle>{bundles})Bundle[^ \n\t\v\r:;`]*?['"])"#
            r#"|(?P<std>['"](?P<root>%kernel\.project_dir%/)?(?P<path>(?:(?:\.|\.\.|[A-Za-z\-_0-9]*)/)+)(?P<file_name>[A-Za-z\-_0-9.]*)(?::(?P<action>[A-Za-z\-_0-9]*))?['"])"#,
        ),
        bundles = bundles_reg_str,
        root_nspace = G.root_namespace
    );
    match Regex::new(&reg_str) {
        Ok(r) => r,
        Err(e) => panic!(
            "Could not build regex from bundle name ({}): {}",
            reg_str, e
        ),
    }
}
