use std::path::PathBuf;

use code_analysis::{load_workspace_files, Emmyrc, LuaFileInfo};
use log::{debug, info};

pub fn collect_files(workspaces: &Vec<PathBuf>, emmyrc: &Emmyrc) -> Vec<LuaFileInfo> {
    let mut files = Vec::new();
    let (match_pattern, exclude) = calculate_include_and_exclude(emmyrc);

    let encoding = &emmyrc.workspace.encoding;

    info!(
        "collect_files from: {:?} match_pattern: {:?} exclude: {:?}",
        workspaces, match_pattern, exclude
    );
    for workspace in workspaces {
        let loaded =
            load_workspace_files(&workspace, &match_pattern, &exclude, Some(encoding)).ok();
        if let Some(loaded) = loaded {
            files.extend(loaded);
        }
    }

    info!("load files from workspace count: {:?}", files.len());
    if cfg!(debug_assertions) {
        for file in &files {
            debug!("loaded file: {:?}", file.path);
        }
    }
    files
}

pub fn calculate_include_and_exclude(emmyrc: &Emmyrc) -> (Vec<String>, Vec<String>) {
    let mut include = vec!["**/*.lua".to_string()];
    let mut exclude = Vec::new();

    for extension in &emmyrc.runtime.extensions {
        if extension.starts_with(".") {
            include.push(format!("**/*{}", extension));
        } else if extension.starts_with("*.") {
            include.push(format!("**/{}", extension));
        } else {
            include.push(extension.clone());
        }
    }

    for ignore_glob in &emmyrc.workspace.ignore_globs {
        exclude.push(ignore_glob.clone());
    }
    
    for dir in &emmyrc.workspace.ignore_dir {
        exclude.push(format!("{}/**", dir));
    }

    // remove duplicate
    include.sort();
    include.dedup();

    // remove duplicate
    exclude.sort();
    exclude.dedup();

    (include, exclude)
}
