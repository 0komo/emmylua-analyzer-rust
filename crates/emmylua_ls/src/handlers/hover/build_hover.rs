use emmylua_code_analysis::{
    DbIndex, LuaDeclId, LuaDocument, LuaMember, LuaMemberId, LuaMemberKey, LuaPropertyOwnerId,
    LuaSignatureId, LuaType, LuaTypeDeclId, RenderLevel, SemanticInfo, SemanticModel,
};
use emmylua_parser::{LuaAssignStat, LuaAstNode, LuaSyntaxKind, LuaSyntaxToken, LuaTableField};
use lsp_types::{Hover, HoverContents, MarkedString, MarkupContent};

use emmylua_code_analysis::humanize_type;

use super::{
    hover_builder::HoverBuilder,
    hover_humanize::{hover_const_type, hover_function_type},
};

pub fn build_semantic_info_hover(
    semantic_model: &SemanticModel,
    db: &DbIndex,
    document: &LuaDocument,
    token: LuaSyntaxToken,
    semantic_info: SemanticInfo,
) -> Option<Hover> {
    let typ = semantic_info.clone().typ;
    if semantic_info.property_owner.is_none() {
        return build_hover_without_property(db, document, token, typ);
    }
    let hover_builder = build_hover_content(
        Some(semantic_model),
        db,
        Some(typ),
        semantic_info.property_owner.unwrap(),
        false,
        Some(token.clone()),
    );
    if let Some(hover_builder) = hover_builder {
        hover_builder.build_hover_result(document.to_lsp_range(token.text_range()))
    } else {
        None
    }
}

fn build_hover_without_property(
    db: &DbIndex,
    document: &LuaDocument,
    token: LuaSyntaxToken,
    typ: LuaType,
) -> Option<Hover> {
    let hover = humanize_type(db, &typ, RenderLevel::Detailed);
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: hover,
        }),
        range: document.to_lsp_range(token.text_range()),
    })
}

pub fn build_hover_content<'a>(
    semantic_model: Option<&'a SemanticModel>,
    db: &DbIndex,
    typ: Option<LuaType>,
    property_id: LuaPropertyOwnerId,
    is_completion: bool, // 是否补全
    token: Option<LuaSyntaxToken>,
) -> Option<HoverBuilder<'a>> {
    match property_id {
        LuaPropertyOwnerId::LuaDecl(decl_id) => {
            if let Some(semantic_model) = semantic_model {
                let mut builder = HoverBuilder::new(semantic_model, token, is_completion);
                if let Some(typ) = typ {
                    build_decl_hover(&mut builder, db, typ, decl_id);
                } else {
                    let decl = db.get_decl_index().get_decl(&decl_id)?;
                    let typ = decl.get_type()?;
                    build_decl_hover(&mut builder, db, typ.clone(), decl_id);
                }
                Some(builder)
            } else {
                None
            }
        }
        LuaPropertyOwnerId::Member(member_id) => {
            if let Some(semantic_model) = semantic_model {
                let mut builder = HoverBuilder::new(semantic_model, token, is_completion);
                if let Some(typ) = typ {
                    build_member_hover(&mut builder, db, typ, member_id);
                } else {
                    let member = db.get_member_index().get_member(&member_id)?;
                    let typ = member.get_decl_type();
                    build_member_hover(&mut builder, db, typ.clone(), member_id);
                }
                Some(builder)
            } else {
                None
            }
        }
        LuaPropertyOwnerId::TypeDecl(type_decl_id) => {
            if let Some(semantic_model) = semantic_model {
                let mut builder = HoverBuilder::new(semantic_model, token, is_completion);
                build_type_decl_hover(&mut builder, db, type_decl_id);
                Some(builder)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn build_decl_hover(
    builder: &mut HoverBuilder,
    db: &DbIndex,
    typ: LuaType,
    decl_id: LuaDeclId,
) -> Option<()> {
    let decl = db.get_decl_index().get_decl(&decl_id)?;

    let mut owner_member = None;
    let mut owner_decl = None;

    // 处理类型签名
    if typ.is_function() {
        let property_owner: Option<LuaPropertyOwnerId> =
            get_decl_owner(builder.semantic_model, decl_id.clone());
        match property_owner {
            Some(LuaPropertyOwnerId::Member(member_id)) => {
                owner_member = Some(db.get_member_index().get_member(&member_id).unwrap());
            }
            Some(LuaPropertyOwnerId::LuaDecl(decl_id)) => {
                owner_decl = Some(db.get_decl_index().get_decl(&decl_id).unwrap());
            }
            _ => {}
        }
        hover_function_type(
            builder,
            db,
            &typ,
            owner_member,
            if let Some(owner_decl) = owner_decl {
                owner_decl.get_name()
            } else {
                decl.get_name()
            },
        );

        builder.set_location_path(owner_member);
    } else if typ.is_const() {
        let const_value = hover_const_type(db, &typ);
        let prefix = if decl.is_local() {
            "local "
        } else {
            "(global) "
        };
        builder.set_type_description(format!("{}{}: {}", prefix, decl.get_name(), const_value));
    } else {
        let type_humanize_text = humanize_type(db, &typ, RenderLevel::Detailed);
        let prefix = if decl.is_local() {
            "local "
        } else {
            "(global) "
        };
        builder.set_type_description(format!(
            "{}{}: {}",
            prefix,
            decl.get_name(),
            type_humanize_text
        ));
    }

    builder.add_annotation_description("---".to_string());

    // 如果`decl`没有描述, 则尝试从`owner_member/owner_decl`获取描述
    builder
        .add_description(LuaPropertyOwnerId::LuaDecl(decl_id))
        .or_else(|| {
            owner_member.and_then(|m: &LuaMember| {
                builder.add_description(LuaPropertyOwnerId::Member(m.get_id()))
            })
        })
        .or_else(|| {
            owner_decl
                .and_then(|d| builder.add_description(LuaPropertyOwnerId::LuaDecl(d.get_id())))
        });

    builder.add_signature_params_rets_description(typ);
    Some(())
}

fn build_member_hover(
    builder: &mut HoverBuilder,
    db: &DbIndex,
    typ: LuaType,
    member_id: LuaMemberId,
) -> Option<()> {
    let member = db.get_member_index().get_member(&member_id)?;
    let member_name = match member.get_key() {
        LuaMemberKey::Name(name) => name.to_string(),
        LuaMemberKey::Integer(i) => format!("[{}]", i),
        _ => return None,
    };

    let mut function_member = None;
    let mut owner_decl = None;
    if typ.is_function() {
        let property_owner = get_member_owner(&builder.semantic_model, member_id);
        match property_owner {
            Some(LuaPropertyOwnerId::Member(member_id)) => {
                function_member = Some(db.get_member_index().get_member(&member_id).unwrap());
            }
            Some(LuaPropertyOwnerId::LuaDecl(decl_id)) => {
                owner_decl = Some(db.get_decl_index().get_decl(&decl_id).unwrap());
            }
            _ => {}
        }
        hover_function_type(
            builder,
            db,
            &typ,
            function_member.or_else(|| {
                if owner_decl.is_none() {
                    Some(&member)
                } else {
                    None
                }
            }),
            if let Some(owner_decl) = owner_decl {
                owner_decl.get_name()
            } else {
                &member_name
            },
        );

        builder.set_location_path(Some(&function_member.as_ref().unwrap_or(&member)));
    } else if typ.is_const() {
        let const_value = hover_const_type(db, &typ);
        builder.set_type_description(format!("(field) {}: {}", member_name, const_value));
    } else {
        let type_humanize_text = humanize_type(db, &typ, RenderLevel::Simple);
        builder.set_type_description(format!("(field) {}: {}", member_name, type_humanize_text));
    }

    builder.add_annotation_description("---".to_string());

    // 如果`decl`没有描述, 则从`owner_member`获取描述
    builder
        .add_description(LuaPropertyOwnerId::Member(member_id))
        .or_else(|| {
            function_member
                .and_then(|m| builder.add_description(LuaPropertyOwnerId::Member(m.get_id())))
        });

    builder.add_signature_params_rets_description(typ);
    Some(())
}

fn build_type_decl_hover(
    builder: &mut HoverBuilder,
    db: &DbIndex,
    type_decl_id: LuaTypeDeclId,
) -> Option<()> {
    let type_decl = db.get_type_index().get_type_decl(&type_decl_id)?;
    let type_description;
    if type_decl.is_alias() {
        if let Some(origin) = type_decl.get_alias_origin(db, None) {
            let origin_type = humanize_type(db, &origin, RenderLevel::Detailed);
            type_description = format!("(type alias) {} = {}", type_decl.get_name(), origin_type)
        } else {
            let mut s = String::new();
            s.push_str(&format!("(type alias) {}\n", type_decl.get_name()));
            let member_ids = type_decl.get_alias_union_members()?;
            for member_id in member_ids {
                let member = db.get_member_index().get_member(&member_id)?;
                let type_humanize_text =
                    humanize_type(db, &member.get_decl_type(), RenderLevel::Minimal);
                let property_owner = LuaPropertyOwnerId::Member(member_id.clone());
                let description = db
                    .get_property_index()
                    .get_property(property_owner)
                    .and_then(|p| p.description.clone());
                if let Some(description) = description {
                    s.push_str(&format!(
                        "    | {}  --{}\n",
                        type_humanize_text, description
                    ));
                } else {
                    s.push_str(&format!("    | {}\n", type_humanize_text));
                }
            }
            type_description = s;
        }
    } else if type_decl.is_enum() {
        type_description = format!("(enum) {}", type_decl.get_name());
    } else {
        let humanize_text = humanize_type(
            db,
            &LuaType::Def(type_decl_id.clone()),
            RenderLevel::Detailed,
        );
        type_description = format!("(class) {}", humanize_text);
    }

    builder.set_type_description(type_description);
    builder.add_description(LuaPropertyOwnerId::TypeDecl(type_decl_id));
    Some(())
}

pub fn add_signature_param_description(
    db: &DbIndex,
    marked_strings: &mut Vec<MarkedString>,
    signature_id: LuaSignatureId,
) -> Option<()> {
    let signature = db.get_signature_index().get(&signature_id)?;
    let param_count = signature.params.len();
    let mut s = String::new();
    for i in 0..param_count {
        let param_info = match signature.get_param_info_by_id(i) {
            Some(info) => info,
            None => continue,
        };

        if let Some(description) = &param_info.description {
            s.push_str(&format!(
                "@*param* `{}` — {}\n",
                param_info.name, description
            ));
        }
    }

    if !s.is_empty() {
        marked_strings.push(MarkedString::from_markdown(s));
    }
    Some(())
}

pub fn add_signature_ret_description(
    db: &DbIndex,
    marked_strings: &mut Vec<MarkedString>,
    signature_id: LuaSignatureId,
) -> Option<()> {
    let signature = db.get_signature_index().get(&signature_id)?;
    let mut s = String::new();
    for i in 0..signature.return_docs.len() {
        let ret_info = &signature.return_docs[i];
        if let Some(description) = ret_info.description.clone() {
            s.push_str(&format!(
                "@*return* {} — {}\n",
                match &ret_info.name {
                    Some(name) if !name.is_empty() => format!("`{}` ", name),
                    _ => "".to_string(),
                },
                description
            ));
        }
    }
    marked_strings.push(MarkedString::from_markdown(s));
    Some(())
}

// 获取`decl`可能的来源
fn get_decl_owner(
    semantic_model: &SemanticModel,
    decl_id: LuaDeclId,
) -> Option<LuaPropertyOwnerId> {
    let root = semantic_model
        .get_db()
        .get_vfs()
        .get_syntax_tree(&decl_id.file_id)?
        .get_red_root();
    let node = semantic_model
        .get_db()
        .get_decl_index()
        .get_decl(&decl_id)?
        .get_value_syntax_id()?
        .to_node_from_root(&root)?;
    let property_owner = semantic_model.get_property_owner_id(node.into());
    match property_owner {
        Some(LuaPropertyOwnerId::Member(member_id)) => get_member_owner(semantic_model, member_id),
        Some(LuaPropertyOwnerId::LuaDecl(_)) => property_owner,
        _ => None,
    }
}

// 获取`member_id`可能的来源
fn get_member_owner(
    semantic_model: &SemanticModel,
    member_id: LuaMemberId,
) -> Option<LuaPropertyOwnerId> {
    fn resolve_owner(
        semantic_model: &SemanticModel,
        member_id: LuaMemberId,
    ) -> Option<LuaPropertyOwnerId> {
        let root = semantic_model
            .get_db()
            .get_vfs()
            .get_syntax_tree(&member_id.file_id)?
            .get_red_root();
        let cur_node = member_id.get_syntax_id().to_node_from_root(&root)?;

        match member_id.get_syntax_id().get_kind() {
            LuaSyntaxKind::TableFieldAssign => match cur_node {
                table_field_node if LuaTableField::can_cast(table_field_node.kind().into()) => {
                    let table_field = LuaTableField::cast(table_field_node)?;
                    let value_expr_syntax_id = table_field.get_value_expr()?.get_syntax_id();
                    let expr = value_expr_syntax_id.to_node_from_root(&root)?;
                    semantic_model.get_property_owner_id(expr.clone().into())
                }
                _ => None,
            },
            LuaSyntaxKind::IndexExpr => {
                let assign_node = cur_node.parent()?;
                match assign_node {
                    assign_node if LuaAssignStat::can_cast(assign_node.kind().into()) => {
                        let assign_stat = LuaAssignStat::cast(assign_node)?;
                        let (vars, exprs) = assign_stat.get_var_and_expr_list();
                        let mut property_owner = None;
                        for (var, expr) in vars.iter().zip(exprs.iter()) {
                            if var.syntax().text_range() == cur_node.text_range() {
                                let expr = expr.get_syntax_id().to_node_from_root(&root)?;
                                property_owner =
                                    semantic_model.get_property_owner_id(expr.clone().into())
                            } else {
                                property_owner = None;
                            }
                        }
                        property_owner
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    let mut current_property_owner = resolve_owner(semantic_model, member_id);
    let mut resolved_property_owner = current_property_owner.clone();
    while let Some(property_owner) = &current_property_owner {
        match property_owner {
            LuaPropertyOwnerId::Member(member_id) => {
                if let Some(next_property_owner) = resolve_owner(semantic_model, member_id.clone())
                {
                    resolved_property_owner = Some(next_property_owner.clone());
                    current_property_owner = Some(next_property_owner.clone());
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    resolved_property_owner
}


