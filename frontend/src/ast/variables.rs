use colored::Colorize;

use crate::{
    ast::{
        GeneratorOutputType, generate_ast, is_valid_identifier,
        types::{ASTBlockType, VariableDefinition},
    },
    errors::SpannedError,
    tokens_parser::types::UVParseNode,
    types::{Positional, Spanned},
};

/// Parse definition of variables <let>
pub fn parse_var_definition(node: &UVParseNode) -> GeneratorOutputType {
    let extra = node.search_extra_children(vec!["name", "value", "const"]);
    if !extra.is_empty() {
        let first = extra.first().unwrap();
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

    let name = name_block.get_inner_literal().unwrap(); // This unwrap is unreachable due checks above

    if !is_valid_identifier(&name.value) {
        return Err(SpannedError::new(
            format!("`{}` is not a valid name for variable", name.value),
            name.span,
        ));
    }

    let value_block = node
        .get_child_by_name("value")
        .ok_or(SpannedError::new("Variable must be initialized", node.span))?;

    if value_block.children_len() != 1 || !value_block.all_tags() {
        return Err(SpannedError::new(
            format!(
                "Variable value must have only one inner tag.\n{}{}",
                "tip".green(),
                ": If you want to place multiple tags, wrap them in a <b> tag.",
            ),
            value_block.span,
        ));
    }

    let value = value_block.get_child_node(0).unwrap(); // This unwrap is unreachable due checks above
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

    Ok(ASTBlockType::VariableDefinition(VariableDefinition {
        name: Spanned::new(name.value.clone(), name_block.span),
        value: Spanned::new(Box::new(generate_ast(value)?), value_block.span),
        is_const: is_const,
        span: node.span,
    }))
}
