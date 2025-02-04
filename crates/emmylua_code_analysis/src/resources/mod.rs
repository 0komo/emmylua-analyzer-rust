use std::path::PathBuf;

use include_dir::{include_dir, Dir, DirEntry};

use crate::{load_workspace_files, LuaFileInfo};

static RESOURCE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/resources");

pub fn load_resource_std(allow_create_resources_dir: bool) -> (PathBuf, Vec<LuaFileInfo>) {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let resoucres_dir = exe_dir.join("resources");
    let std_dir = resoucres_dir.join("std");

    if allow_create_resources_dir {
        let result = load_resource_from_file_system();
        match result {
            Some(files) => return (std_dir, files),
            None => {}
        }
    }

    let files = load_resource_from_include_dir();
    let files = files
        .into_iter()
        .filter_map(|file| {
            if file.path.ends_with(".lua") {
                let path = std_dir.join(&file.path).to_str().unwrap().to_string();
                Some(LuaFileInfo {
                    path,
                    content: file.content,
                })
            } else {
                None
            }
        })
        .collect::<_>();

    (std_dir, files)
}

fn load_resource_from_file_system() -> Option<Vec<LuaFileInfo>> {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let resoucres_dir = exe_dir.join("resources");

    if !resoucres_dir.exists() {
        log::info!("Creating resources dir: {:?}", resoucres_dir);
        let files = load_resource_from_include_dir();
        for file in &files {
            let path = resoucres_dir.join(&file.path);
            let parent = path.parent().unwrap();
            if !parent.exists() {
                match std::fs::create_dir_all(parent) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Failed to create dir: {:?}, {:?}", parent, e);
                        return None;
                    }
                }
            }

            match std::fs::write(&path, &file.content) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to write file: {:?}, {:?}", path, e);
                    return None;
                }
            }
        }
    }

    let std_dir = resoucres_dir.join("std");
    let match_pattern = vec!["**/*.lua".to_string()];
    let files = match load_workspace_files(&std_dir, &match_pattern, &Vec::new(), &Vec::new(), None)
    {
        Ok(files) => files,
        Err(e) => {
            log::error!("Failed to load std lib: {:?}", e);
            vec![]
        }
    };

    return Some(files);
}

fn load_resource_from_include_dir() -> Vec<LuaFileInfo> {
    let mut files = Vec::new();
    walk_resource_dir(&RESOURCE_DIR, &mut files);
    files
}

fn walk_resource_dir(dir: &Dir, files: &mut Vec<LuaFileInfo>) {
    for entry in dir.entries() {
        match entry {
            DirEntry::File(file) => {
                let path = file.path();
                let content = file.contents_utf8().unwrap();

                files.push(LuaFileInfo {
                    path: path.to_str().unwrap().to_string(),
                    content: content.to_string(),
                });
            }
            DirEntry::Dir(subdir) => {
                walk_resource_dir(subdir, files);
            }
        }
    }
}
