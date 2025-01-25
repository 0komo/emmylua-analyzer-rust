use std::path::Path;

use code_analysis::{
    humanize_type, DbIndex, LuaMemberKey, LuaMemberOwner, LuaPropertyOwnerId, LuaTypeDecl,
};
use emmylua_parser::VisibilityKind;
use tera::Tera;

use crate::markdown_generator::{escape_type_name, IndexStruct, MemberDisplay};

use super::{
    render::{render_const_type, render_function_type},
    MkdocsIndex,
};

pub fn generate_type_markdown(
    db: &DbIndex,
    tl: &Tera,
    typ: &LuaTypeDecl,
    input: &Path,
    output: &Path,
    mkdocs_index: &mut MkdocsIndex,
) -> Option<()> {
    check_filter(db, typ, input)?;

    let mut context = tera::Context::new();
    let typ_name = typ.get_name();
    context.insert("type_name", &typ_name);

    let typ_id = typ.get_id();
    let namespace = typ.get_namespace();
    context.insert("namespace", &namespace);

    let type_property_id = LuaPropertyOwnerId::TypeDecl(typ_id.clone());
    let typ_property = db.get_property_index().get_property(type_property_id);
    if let Some(typ_property) = typ_property {
        if let Some(property_text) = &typ_property.description {
            context.insert("description", &property_text);
        }
    }

    let supers = db.get_type_index().get_super_types(&typ_id);
    if let Some(supers) = supers {
        let mut super_type_texts = Vec::new();
        for super_typ in supers {
            let super_type_text = humanize_type(db, &super_typ);
            super_type_texts.push(super_type_text);
        }
        context.insert("super_types", &super_type_texts.join(", "));
    }

    let member_owner = LuaMemberOwner::Type(typ_id);
    let member_map = db.get_member_index().get_member_map(member_owner);
    let mut method_members: Vec<MemberDisplay> = Vec::new();
    let mut field_members: Vec<MemberDisplay> = Vec::new();
    if let Some(member_map) = member_map {
        for (member_name, member_id) in member_map {
            let member = db.get_member_index().get_member(member_id)?;
            let member_typ = member.get_decl_type();
            let member_property_id = LuaPropertyOwnerId::Member(member_id.clone());
            let member_property = db.get_property_index().get_property(member_property_id);
            if let Some(member_property) = member_property {
                if member_property.visibility.unwrap_or(VisibilityKind::Public)
                    != VisibilityKind::Public
                {
                    continue;
                }
            }

            let description = if let Some(member_property) = member_property {
                *member_property
                    .description
                    .clone()
                    .unwrap_or("".to_string().into())
            } else {
                "".to_string()
            };

            let name = match member_name {
                LuaMemberKey::Name(name) => name,
                _ => continue,
            };

            let title_name = format!("{}.{}", typ_name, name);
            if member_typ.is_function() {
                let func_name = format!("{}.{}", typ_name, name);
                let display = render_function_type(db, member_typ, &func_name, false);
                method_members.push(MemberDisplay {
                    name: title_name,
                    display,
                    description,
                });
            } else if member_typ.is_const() {
                let const_type_display = render_const_type(db, &member_typ);
                field_members.push(MemberDisplay {
                    name: title_name,
                    display: format!("```lua\n{}.{}: {}\n```\n", typ_name, name, const_type_display),
                    description,
                });
            } else {
                let typ_display = humanize_type(db, &member_typ);
                field_members.push(MemberDisplay {
                    name: title_name,
                    display: format!("```lua\n{}.{} : {}\n```\n", typ_name, name, typ_display),
                    description,
                });
            }
        }
    }
    if !method_members.is_empty() {
        context.insert("methods", &method_members);
    }

    if !field_members.is_empty() {
        context.insert("fields", &field_members);
    }

    let render_text = match tl.render("lua_type_template.tl", &context) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Failed to render template: {}", e);
            return None;
        }
    };

    let file_type_name = format!("{}.md", escape_type_name(typ.get_full_name()));
    mkdocs_index.types.push(IndexStruct {
        name: typ_name.to_string(),
        file: format!("types/{}", file_type_name.clone()),
    });

    let outpath = output.join(file_type_name);
    println!("output type file: {}", outpath.display());
    match std::fs::write(outpath, render_text) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to write file: {}", e);
            return None;
        }
    }
    Some(())
}

fn check_filter(db: &DbIndex, typ: &LuaTypeDecl, workspace: &Path) -> Option<()> {
    let location = typ.get_locations();
    for loc in location {
        let file_id = loc.file_id;
        let file_path = db.get_vfs().get_file_path(&file_id)?;
        if !file_path.starts_with(workspace) {
            return None;
        }
    }

    Some(())
}