use std::collections::HashSet;

use emmylua_code_analysis::{LuaMemberKey, LuaPropertyOwnerId, LuaType, SemanticModel};
use emmylua_parser::{
    LuaAst, LuaAstNode, LuaAstToken, LuaDocFieldKey, LuaDocObjectFieldKey, LuaExpr, LuaNameToken,
    LuaSyntaxNode, LuaSyntaxToken, LuaTokenKind, LuaVarExpr,
};
use lsp_types::{SemanticToken, SemanticTokenModifier, SemanticTokenType};
use rowan::{NodeOrToken, TextRange};

use crate::context::ClientId;

use super::{
    semantic_token_builder::SemanticBuilder, SEMANTIC_TOKEN_MODIFIERS, SEMANTIC_TOKEN_TYPES,
};

pub fn build_semantic_tokens(
    semantic_model: &SemanticModel,
    support_muliline_token: bool,
    client_id: ClientId,
) -> Option<Vec<SemanticToken>> {
    let root = semantic_model.get_root();
    let document = semantic_model.get_document();
    let mut builder = SemanticBuilder::new(
        &document,
        support_muliline_token,
        SEMANTIC_TOKEN_TYPES.to_vec(),
        SEMANTIC_TOKEN_MODIFIERS.to_vec(),
    );

    // 用于渲染全局方法/标准库
    let mut use_range_set = HashSet::new();
    calc_name_expr_ref(semantic_model, &mut use_range_set);

    for node_or_token in root.syntax().descendants_with_tokens() {
        match node_or_token {
            NodeOrToken::Node(node) => {
                build_node_semantic_token(
                    semantic_model,
                    &mut builder,
                    node,
                    &mut use_range_set,
                    client_id,
                );
            }
            NodeOrToken::Token(token) => {
                build_tokens_semantic_token(semantic_model, &mut builder, token, client_id);
            }
        }
    }

    Some(builder.build())
}

#[allow(unused)]
fn build_tokens_semantic_token(
    semantic_model: &SemanticModel,
    builder: &mut SemanticBuilder,
    token: LuaSyntaxToken,
    client_id: ClientId,
) {
    match token.kind().into() {
        LuaTokenKind::TkLongString | LuaTokenKind::TkString => {
            builder.push(token, SemanticTokenType::STRING);
        }
        LuaTokenKind::TkAnd
        | LuaTokenKind::TkBreak
        | LuaTokenKind::TkDo
        | LuaTokenKind::TkElse
        | LuaTokenKind::TkElseIf
        | LuaTokenKind::TkEnd
        | LuaTokenKind::TkFor
        | LuaTokenKind::TkFunction
        | LuaTokenKind::TkGoto
        | LuaTokenKind::TkIf
        | LuaTokenKind::TkIn
        | LuaTokenKind::TkNot
        | LuaTokenKind::TkOr
        | LuaTokenKind::TkRepeat
        | LuaTokenKind::TkReturn
        | LuaTokenKind::TkThen
        | LuaTokenKind::TkUntil
        | LuaTokenKind::TkWhile => {
            builder.push(token, SemanticTokenType::KEYWORD);
        }
        LuaTokenKind::TkLocal => {
            if !client_id.is_vscode() {
                builder.push(token, SemanticTokenType::KEYWORD);
            }
        }
        LuaTokenKind::TkPlus
        | LuaTokenKind::TkMinus
        | LuaTokenKind::TkMul
        | LuaTokenKind::TkDiv
        | LuaTokenKind::TkIDiv
        | LuaTokenKind::TkDot
        | LuaTokenKind::TkConcat
        | LuaTokenKind::TkEq
        | LuaTokenKind::TkGe
        | LuaTokenKind::TkLe
        | LuaTokenKind::TkNe
        | LuaTokenKind::TkShl
        | LuaTokenKind::TkShr
        | LuaTokenKind::TkLt
        | LuaTokenKind::TkGt
        | LuaTokenKind::TkMod
        | LuaTokenKind::TkPow
        | LuaTokenKind::TkLen
        | LuaTokenKind::TkBitAnd
        | LuaTokenKind::TkBitOr
        | LuaTokenKind::TkBitXor
        | LuaTokenKind::TkLeftBrace
        | LuaTokenKind::TkRightBrace
        | LuaTokenKind::TkLeftBracket
        | LuaTokenKind::TkRightBracket => {
            builder.push(token, SemanticTokenType::OPERATOR);
        }
        LuaTokenKind::TkComplex | LuaTokenKind::TkInt | LuaTokenKind::TkFloat => {
            builder.push(token, SemanticTokenType::NUMBER);
        }
        LuaTokenKind::TkTagClass
        | LuaTokenKind::TkTagEnum
        | LuaTokenKind::TkTagInterface
        | LuaTokenKind::TkTagAlias
        | LuaTokenKind::TkTagModule
        | LuaTokenKind::TkTagField
        | LuaTokenKind::TkTagType
        | LuaTokenKind::TkTagParam
        | LuaTokenKind::TkTagReturn
        | LuaTokenKind::TkTagOverload
        | LuaTokenKind::TkTagGeneric
        | LuaTokenKind::TkTagSee
        | LuaTokenKind::TkTagDeprecated
        | LuaTokenKind::TkTagAsync
        | LuaTokenKind::TkTagCast
        | LuaTokenKind::TkTagOther
        | LuaTokenKind::TkTagReadonly
        | LuaTokenKind::TkTagDiagnostic
        | LuaTokenKind::TkTagMeta
        | LuaTokenKind::TkTagVersion
        | LuaTokenKind::TkTagAs
        | LuaTokenKind::TkTagNodiscard
        | LuaTokenKind::TkTagOperator
        | LuaTokenKind::TkTagMapping
        | LuaTokenKind::TkTagNamespace
        | LuaTokenKind::TkTagUsing
        | LuaTokenKind::TkTagSource => {
            builder.push_with_modifier(
                token,
                SemanticTokenType::KEYWORD,
                SemanticTokenModifier::DOCUMENTATION,
            );
        }
        LuaTokenKind::TkDocKeyOf
        | LuaTokenKind::TkDocExtends
        | LuaTokenKind::TkDocAs
        | LuaTokenKind::TkDocIn
        | LuaTokenKind::TkDocInfer => {
            builder.push(token, SemanticTokenType::KEYWORD);
        }
        LuaTokenKind::TkDocDetail => {
            builder.push(token, SemanticTokenType::COMMENT);
        }
        LuaTokenKind::TkDocQuestion => {
            builder.push(token, SemanticTokenType::OPERATOR);
        }
        LuaTokenKind::TkDocVisibility | LuaTokenKind::TkTagVisibility => {
            builder.push_with_modifier(
                token,
                SemanticTokenType::KEYWORD,
                SemanticTokenModifier::MODIFICATION,
            );
        }
        LuaTokenKind::TkDocVersionNumber => {
            builder.push(token, SemanticTokenType::NUMBER);
        }
        LuaTokenKind::TkStringTemplateType => {
            builder.push(token, SemanticTokenType::STRING);
        }
        LuaTokenKind::TkDocMatch | LuaTokenKind::TkDocBoolean => {
            builder.push(token, SemanticTokenType::KEYWORD);
        }
        LuaTokenKind::TKDocPath => {
            builder.push(token, SemanticTokenType::STRING);
        }
        LuaTokenKind::TkDocRegion | LuaTokenKind::TkDocEndRegion => {
            builder.push(token, SemanticTokenType::KEYWORD);
        }
        LuaTokenKind::TkDocStart => {
            let range = token.text_range();
            // find '@'
            let text = token.text();
            let mut start = 0;
            for (i, c) in text.char_indices() {
                if c == '@' {
                    start = i;
                    break;
                }
            }
            let position = u32::from(range.start()) + start as u32;
            builder.push_at_position(
                position.into(),
                1,
                SemanticTokenType::KEYWORD,
                SemanticTokenModifier::DOCUMENTATION,
            );
        }
        // LuaTokenKind::TkName => {
        //     let property_owner = semantic_model.get_property_owner_id(token.clone().into());
        //     match property_owner {
        //         Some(LuaPropertyOwnerId::LuaDecl(decl_id)) => {
        //             let decl = semantic_model.get_db().get_decl_index().get_decl(&decl_id);
        //             if let Some(decl) = decl {
        //                 let decl_type = decl.get_type();
        //                 if let Some(decl_type) = decl_type {
        //                     match decl_type {
        //                         LuaType::Def(def) => {
        //                             builder.push(token, SemanticTokenType::CLASS);
        //                         }
        //                         _ => {}
        //                     }
        //                 }
        //             }
        //         }
        //         _ => {}
        //     }
        // }
        _ => {}
    }
}

fn build_node_semantic_token(
    semantic_model: &SemanticModel,
    builder: &mut SemanticBuilder,
    node: LuaSyntaxNode,
    use_range_set: &mut HashSet<TextRange>,
    _: ClientId,
) -> Option<()> {
    match LuaAst::cast(node)? {
        LuaAst::LuaDocTagClass(doc_class) => {
            let name = doc_class.get_name_token()?;
            builder.push_with_modifier(
                name.syntax().clone(),
                SemanticTokenType::CLASS,
                SemanticTokenModifier::DECLARATION,
            );
            if let Some(attribs) = doc_class.get_attrib() {
                for attrib_token in attribs.get_attrib_tokens() {
                    builder.push(attrib_token.syntax().clone(), SemanticTokenType::MODIFIER);
                }
            }
            if let Some(generic_list) = doc_class.get_generic_decl() {
                for generic_decl in generic_list.get_generic_decl() {
                    if let Some(name) = generic_decl.get_name_token() {
                        builder.push_with_modifier(
                            name.syntax().clone(),
                            SemanticTokenType::CLASS,
                            SemanticTokenModifier::DECLARATION,
                        );
                    }
                }
            }
        }
        LuaAst::LuaDocTagEnum(doc_enum) => {
            let name = doc_enum.get_name_token()?;
            builder.push(name.syntax().clone(), SemanticTokenType::ENUM);
            if let Some(attribs) = doc_enum.get_attrib() {
                for attrib_token in attribs.get_attrib_tokens() {
                    builder.push(attrib_token.syntax().clone(), SemanticTokenType::MODIFIER);
                }
            }
        }
        LuaAst::LuaDocTagAlias(doc_alias) => {
            let name = doc_alias.get_name_token()?;
            builder.push(name.syntax().clone(), SemanticTokenType::TYPE);
        }
        LuaAst::LuaDocTagField(doc_field) => {
            if let Some(LuaDocFieldKey::Name(name)) = doc_field.get_field_key() {
                builder.push(name.syntax().clone(), SemanticTokenType::PROPERTY);
            }
        }
        LuaAst::LuaDocTagDiagnostic(doc_diagnostic) => {
            let name = doc_diagnostic.get_action_token()?;
            builder.push(name.syntax().clone(), SemanticTokenType::PROPERTY);
            if let Some(code_list) = doc_diagnostic.get_code_list() {
                for code in code_list.get_codes() {
                    builder.push(code.syntax().clone(), SemanticTokenType::REGEXP);
                }
            }
        }
        LuaAst::LuaDocTagParam(doc_param) => {
            let name = doc_param.get_name_token()?;
            builder.push(name.syntax().clone(), SemanticTokenType::PARAMETER);
        }
        LuaAst::LuaDocTagReturn(doc_return) => {
            let type_name_list = doc_return.get_type_and_name_list();
            for (_, name) in type_name_list {
                if let Some(name) = name {
                    builder.push(name.syntax().clone(), SemanticTokenType::VARIABLE);
                }
            }
        }
        LuaAst::LuaDocTagCast(doc_cast) => {
            let name = doc_cast.get_name_token()?;
            builder.push(name.syntax().clone(), SemanticTokenType::VARIABLE);
        }
        LuaAst::LuaDocTagGeneric(doc_generic) => {
            let type_parameter_list = doc_generic.get_generic_decl_list()?;
            for type_decl in type_parameter_list.get_generic_decl() {
                if let Some(name) = type_decl.get_name_token() {
                    builder.push_with_modifier(
                        name.syntax().clone(),
                        SemanticTokenType::TYPE,
                        SemanticTokenModifier::DECLARATION,
                    );
                }
            }
        }
        LuaAst::LuaDocTagNamespace(doc_namespace) => {
            let name = doc_namespace.get_name_token()?;
            builder.push(name.syntax().clone(), SemanticTokenType::NAMESPACE);
        }
        LuaAst::LuaDocTagUsing(doc_using) => {
            let name = doc_using.get_name_token()?;
            builder.push(name.syntax().clone(), SemanticTokenType::NAMESPACE);
        }
        LuaAst::LuaParamName(param_name) => {
            let name = param_name.get_name_token()?;
            builder.push(name.syntax().clone(), SemanticTokenType::PARAMETER);
        }
        LuaAst::LuaLocalName(local_name) => {
            let name = local_name.get_name_token()?;
            if let Some(true) = is_class_def(semantic_model, local_name.syntax().clone()) {
                builder.push(name.syntax().clone(), SemanticTokenType::CLASS);
                return Some(());
            }
            builder.push(name.syntax().clone(), SemanticTokenType::VARIABLE);
        }
        LuaAst::LuaNameExpr(name_expr) => {
            let name = name_expr.get_name_token()?;
            if let Some(true) = is_class_def(semantic_model, name_expr.syntax().clone()) {
                builder.push(name.syntax().clone(), SemanticTokenType::CLASS);
                return Some(());
            }
            builder.push(name.syntax().clone(), SemanticTokenType::VARIABLE);
        }
        LuaAst::LuaForRangeStat(for_range_stat) => {
            for name in for_range_stat.get_var_name_list() {
                builder.push(name.syntax().clone(), SemanticTokenType::VARIABLE);
            }
        }
        LuaAst::LuaForStat(for_stat) => {
            let name = for_stat.get_var_name()?;
            builder.push(name.syntax().clone(), SemanticTokenType::VARIABLE);
        }
        LuaAst::LuaLocalFuncStat(local_func_stat) => {
            let name = local_func_stat.get_local_name()?.get_name_token()?;
            builder.push(name.syntax().clone(), SemanticTokenType::FUNCTION);
        }
        LuaAst::LuaFuncStat(func_stat) => {
            let func_name = func_stat.get_func_name()?;
            match func_name {
                LuaVarExpr::NameExpr(name_expr) => {
                    let name = name_expr.get_name_token()?;
                    builder.push(name.syntax().clone(), SemanticTokenType::FUNCTION);
                }
                LuaVarExpr::IndexExpr(index_expr) => {
                    let name = index_expr.get_index_name_token()?;
                    builder.push(name, SemanticTokenType::FUNCTION);
                }
            }
        }
        LuaAst::LuaLocalStat(local_stat) => {
            let var_exprs = local_stat.descendants::<LuaVarExpr>();
            for var_expr in var_exprs {
                match var_expr {
                    LuaVarExpr::IndexExpr(name_expr) => {
                        let name = name_expr.get_name_token()?;
                        builder.push(name.syntax().clone(), SemanticTokenType::PROPERTY);
                    }
                    _ => {}
                }
            }
        }
        LuaAst::LuaLocalAttribute(local_attribute) => {
            let name = local_attribute.get_name_token()?;
            builder.push(name.syntax().clone(), SemanticTokenType::KEYWORD);
        }
        LuaAst::LuaCallExpr(call_expr) => {
            let prefix = call_expr.get_prefix_expr()?;
            let prefix_type = semantic_model.infer_expr(prefix.clone());

            match prefix {
                LuaExpr::NameExpr(name_expr) => {
                    let name = name_expr.get_name_token()?;

                    if let Some(prefix_type) = prefix_type {
                        match prefix_type {
                            LuaType::Signature(_) => {
                                let name_text = name_expr.get_name_text()?;
                                let name_range = name_expr.get_range();
                                if !use_range_set.contains(&name_range) {
                                    let decl_index = semantic_model.get_db().get_decl_index();
                                    let member_key = LuaMemberKey::Name(name_text.clone().into());
                                    if decl_index.get_global_decl_id(&member_key).is_some() {
                                        builder.push_with_modifier(
                                            name.syntax().clone(),
                                            SemanticTokenType::FUNCTION,
                                            SemanticTokenModifier::DEFAULT_LIBRARY,
                                        );
                                        return Some(());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    builder.push(name.syntax().clone(), SemanticTokenType::FUNCTION);
                }
                LuaExpr::IndexExpr(index_expr) => {
                    let name = index_expr.get_index_name_token()?;
                    builder.push(name, SemanticTokenType::FUNCTION);
                }
                _ => {}
            }
        }
        LuaAst::LuaDocNameType(doc_name_type) => {
            let name = doc_name_type.get_name_token()?;
            if name.get_name_text() == "self" {
                builder.push_with_modifier(
                    name.syntax().clone(),
                    SemanticTokenType::TYPE,
                    SemanticTokenModifier::READONLY,
                );
            } else {
                builder.push(name.syntax().clone(), SemanticTokenType::TYPE);
            }
        }
        LuaAst::LuaDocObjectType(doc_object_type) => {
            let fields = doc_object_type.get_fields();
            for field in fields {
                if let Some(field_key) = field.get_field_key() {
                    match &field_key {
                        LuaDocObjectFieldKey::Name(name) => {
                            builder.push(name.syntax().clone(), SemanticTokenType::PROPERTY);
                        }
                        _ => {}
                    }
                }
            }
        }
        LuaAst::LuaDocFuncType(doc_func_type) => {
            for name_token in doc_func_type.tokens::<LuaNameToken>() {
                match name_token.get_name_text() {
                    "fun" => {
                        builder.push(name_token.syntax().clone(), SemanticTokenType::KEYWORD);
                    }
                    "async" => {
                        builder.push_with_modifier(
                            name_token.syntax().clone(),
                            SemanticTokenType::KEYWORD,
                            SemanticTokenModifier::ASYNC,
                        );
                    }
                    _ => {}
                }
            }

            for param in doc_func_type.get_params() {
                let name = param.get_name_token()?;
                builder.push(name.syntax().clone(), SemanticTokenType::PARAMETER);
            }
        }
        LuaAst::LuaIndexExpr(index_expr) => {
            let name = index_expr.get_name_token()?;
            builder.push(name.syntax().clone(), SemanticTokenType::PROPERTY);
        }
        _ => {}
    }

    Some(())
}

fn is_class_def(semantic_model: &SemanticModel, node: LuaSyntaxNode) -> Option<bool> {
    let property_owner = semantic_model.get_property_owner_id(node.into())?;

    if let LuaPropertyOwnerId::LuaDecl(decl_id) = property_owner {
        let decl = semantic_model
            .get_db()
            .get_decl_index()
            .get_decl(&decl_id)?
            .get_type()?;
        match decl {
            LuaType::Def(_) => Some(true),
            _ => None,
        }
    } else {
        None
    }
}

fn calc_name_expr_ref(
    semantic_model: &SemanticModel,
    use_range_set: &mut HashSet<TextRange>,
) -> Option<()> {
    let file_id = semantic_model.get_file_id();
    let db = semantic_model.get_db();
    let refs_index = db.get_reference_index().get_local_reference(&file_id)?;
    for (_, decl_refs) in refs_index.get_decl_references_map() {
        for decl_ref in decl_refs {
            use_range_set.insert(decl_ref.range.clone());
        }
    }

    None
}
