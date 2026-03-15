use std::ops::Deref;

use crate::{
    ast::{
        GeneratorOutputType, generate_ast, is_valid_identifier,
        type_parser::parse_type,
        types::{ASTBlockType, VariableAccess, VariableAssign, VariableDefinition},
    },
    errors::SpannedError,
    tokens_parser::types::UVParseNode,
    types::{Positional, Spanned},
};

/// Parse definition of variables <let>
pub fn parse_var_definition(node: &UVParseNode) -> GeneratorOutputType {
    let extra = node.search_extra_children(vec!["name", "value", "const", "type"]);
    if !extra.is_empty() {
        let first = extra.first().ok_or(SpannedError::new(
            "[INTERNAL ERROR] Cannot get first extra child",
            node.span,
        ))?;

        return Err(SpannedError::new(
            "Found extra children for variable definition",
            first.get_span(),
        ));
    }

    let name_block = node.get_child_by_name("name").ok_or(SpannedError::new(
        "Variable definition should have an inner <name> tag",
        node.span,
    ))?;

    if name_block.children_len() != 1 || !name_block.all_literals() {
        return Err(SpannedError::new("Invalid variable name", name_block.span));
    }

    let name = name_block.get_inner_literal().ok_or(SpannedError::new(
        "[INTERNAL ERROR] Cannot get inner literal",
        node.span,
    ))?;

    if !is_valid_identifier(&name) {
        return Err(SpannedError::new(
            format!("`{}` is not a valid name for variable", name.deref()),
            name.span,
        ));
    }

    let value_block = node
        .get_child_by_name("value")
        .ok_or(SpannedError::new("Variable must be initialized", node.span))?;

    if value_block.children_len() != 1 || !value_block.all_tags() {
        return Err(SpannedError::new_tipped(
            "Variable value must have only one inner tag.",
            "If you want to place multiple tags, wrap them in a <g> tag.",
            value_block.span,
        ));
    }

    let value = value_block.get_tag_at(0).ok_or(SpannedError::new(
        "[INTERNAL ERROR] Cannot get inner tag",
        node.span,
    ))?;

    // <const /> tag
    let is_const = match node.get_child_by_name("const") {
        Some(c) if !c.self_closing => {
            return Err(SpannedError::new(
                "`const` tag must be self-closing",
                c.span,
            ));
        }
        Some(_) => true,
        None => false,
    };

    // Expected type <type>
    let exp_type = match node.get_child_by_name("type") {
        Some(c) if c.self_closing => {
            return Err(SpannedError::new(
                "`type` tag cannot be self-closing",
                c.span,
            ));
        }
        Some(ch) if ch.children_len() != 1 || !ch.all_tags() => {
            return Err(SpannedError::new(
                "`type` tag must contain only one child, representing variable type",
                ch.span,
            ));
        }
        Some(ch) => Some(Spanned::new(
            parse_type(ch.get_tag_at(0).ok_or(SpannedError::new(
                "[INTERNAL ERROR] Cannot get inner child",
                ch.span,
            ))?)?,
            ch.span,
        )),
        None => None,
    };

    Ok(ASTBlockType::VariableDefinition(Box::new(
        VariableDefinition {
            name: Spanned::new(name.deref().clone(), name_block.span),
            value: Spanned::new(generate_ast(value)?, value_block.span),
            expected_type: exp_type,
            is_const: is_const,
            span: node.span,
        },
    )))
}

/// Parse variable assignment
pub fn parse_var_assign(node: &UVParseNode) -> GeneratorOutputType {
    if !node.all_tags() {
        let unexpected_lit = node.get_inner_literal().ok_or(SpannedError::new(
            "[INTERNAL ERROR] Cannot get inner literal for error",
            node.span,
        ))?;

        return Err(SpannedError::new(
            "Cannot assign literal to a variable",
            unexpected_lit.span,
        ));
    }

    if node.children_len() != 1 {
        let extra = node.get_child_at(1);

        return Err(SpannedError::new(
            "Variable assign should have only one nested tag",
            match extra {
                Some(x) => x.get_span(),
                None => node.span,
            },
        ));
    }

    let value = node
        .get_tag_at(0)
        .ok_or(SpannedError::new("Cannot get inner tag", node.span))?;

    Ok(ASTBlockType::VariableAssignment(VariableAssign {
        name: node.name.clone(),
        value: Spanned::new(Box::new(generate_ast(value)?), value.span),
        span: node.span,
    }))
}

/// Parse variable access block
pub fn parse_var_access(node: &UVParseNode) -> GeneratorOutputType {
    if !node.self_closing {
        return Err(SpannedError::new(
            "Variable access block should be self-closing",
            node.span,
        ));
    }

    Ok(ASTBlockType::VariableAccess(VariableAccess {
        name: node.name.clone(),
        span: node.span,
    }))
}
