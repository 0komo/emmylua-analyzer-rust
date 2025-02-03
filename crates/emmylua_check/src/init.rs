use std::{path::PathBuf, sync::Arc};

use emmylua_code_analysis::{
    load_configs, load_workspace_files, EmmyLuaAnalysis, Emmyrc, LuaFileInfo,
};

#[allow(unused)]
pub fn load_workspace(
    workspace_folder: PathBuf,
    config_path: Option<PathBuf>,
    ignore: Option<Vec<String>>,
) -> Option<EmmyLuaAnalysis> {
    let mut analysis = EmmyLuaAnalysis::new();
    analysis.init_std_lib(false);

    let workspace_folders = vec![workspace_folder];
    for path in &workspace_folders {
        analysis.add_workspace_root(path.clone());
    }

    let main_path = workspace_folders.get(0)?.clone();
    let config_files = if let Some(config_path) = config_path {
        vec![config_path]
    } else {
        vec![
            main_path.join(".luarc.json"),
            main_path.join(".emmyrc.json"),
        ]
    };

    let emmyrc = Arc::new(load_configs(config_files, None));
    analysis.update_config(emmyrc);

    let file_infos = collect_files(&workspace_folders, &analysis.emmyrc, ignore);
    let files = file_infos
        .into_iter()
        .map(|file| file.into_tuple())
        .collect();
    analysis.update_files_by_path(files);

    Some(analysis)
}

pub fn collect_files(
    workspaces: &Vec<PathBuf>,
    emmyrc: &Emmyrc,
    ignore: Option<Vec<String>>,
) -> Vec<LuaFileInfo> {
    let mut files = Vec::new();
    let (match_pattern, exclude, exclude_dir) = calculate_include_and_exclude(emmyrc, ignore);

    let encoding = &emmyrc.workspace.encoding;

    for workspace in workspaces {
        let loaded = load_workspace_files(
            &workspace,
            &match_pattern,
            &exclude,
            &exclude_dir,
            Some(encoding),
        )
        .ok();
        if let Some(loaded) = loaded {
            files.extend(loaded);
        }
    }

    files
}

pub fn calculate_include_and_exclude(
    emmyrc: &Emmyrc,
    ignore: Option<Vec<String>>,
) -> (Vec<String>, Vec<String>, Vec<PathBuf>) {
    let mut include = vec!["**/*.lua".to_string()];
    let mut exclude = Vec::new();
    let mut exclude_dirs = Vec::new();

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

    if let Some(ignore) = ignore {
        exclude.extend(ignore);
    }

    for dir in &emmyrc.workspace.ignore_dir {
        exclude_dirs.push(PathBuf::from(dir));
    }

    // remove duplicate
    include.sort();
    include.dedup();

    // remove duplicate
    exclude.sort();
    exclude.dedup();

    (include, exclude, exclude_dirs)
}
