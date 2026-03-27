use std::ops::Deref;

use crate::{
    ast::{
        GeneratorOutputType, is_valid_identifier, parse_children_vec,
        type_parser::validate_and_parse_inner_type_block,
        types::{ASTBlockType, FunctionDefinition, FunctionDefinitionArg},
    },
    errors::SpannedError,
    tokens_parser::{traits::UnwrapOptionError, types::UVParseNode},
    types::Positional,
};

pub fn parse_function_definition(node: &UVParseNode) -> GeneratorOutputType {
    let extra = node.search_extra_children(vec!["name", "arg", "returns", "body"]);

    if !extra.is_empty() {
        let first_extra = extra.first().unwrap_or_spanned(node.span)?;

        return Err(SpannedError::new(
            "Found extra children inside function definition",
            first_extra.get_span(),
        ));
    }

    // ---------------------------- Name ----------------------------

    let name_block = node.get_one_tag_by_name("name").ok_or(SpannedError::new(
        "Function definition should have an inner <name> tag",
        node.span,
    ))?;

    if name_block.children_len() != 1 || !name_block.all_literals() {
        return Err(SpannedError::new("Invalid function name", name_block.span));
    }

    let name = name_block
        .get_inner_literal()
        .unwrap_or_spanned(node.span)?;

    if !is_valid_identifier(&name) {
        return Err(SpannedError::new(
            format!("`{}` is not a valid name for function", name.deref()),
            name.span,
        ));
    }

    // -------------------------- Arguments -------------------------
    let arguments = parse_arguments_definition(node.get_many_tags_by_name("arg"))?;

    // --------------------------- Body -----------------------------

    let body = match node.get_one_tag_by_name("body") {
        Some(x) => x,
        None => {
            return Err(SpannedError::new("Function must have a body", node.span));
        }
    };

    Ok(ASTBlockType::FunctionDefinition(Box::new(
        FunctionDefinition {
            name: name.clone(),
            arguments: arguments,
            return_type: validate_and_parse_inner_type_block(node, "returns")?,
            body: parse_children_vec(body)?,
            span: node.span,
        },
    )))
}

/// Parse function definition arguments
fn parse_arguments_definition(
    args: Vec<&UVParseNode>,
) -> Result<Vec<FunctionDefinitionArg>, SpannedError> {
    args.into_iter()
        .map(|arg| {
            // Name
            let name_block = arg.get_one_tag_by_name("name").ok_or(SpannedError::new(
                "Argument definition should have an inner <name> tag",
                arg.span,
            ))?;

            if name_block.children_len() != 1 || !name_block.all_literals() {
                return Err(SpannedError::new("Invalid argument name", name_block.span));
            }

            let name = name_block.get_inner_literal().unwrap_or_spanned(arg.span)?;

            if !is_valid_identifier(&name) {
                return Err(SpannedError::new(
                    format!("`{}` is not a valid name for argument", name.deref()),
                    name.span,
                ));
            }

            Ok(FunctionDefinitionArg {
                name: name.clone(),
                arg_type: validate_and_parse_inner_type_block(arg, "type")?.ok_or(
                    SpannedError::new("Argument definition should have an `type` tag", arg.span),
                )?,
                span: arg.span,
            })
        })
        .collect()
}
