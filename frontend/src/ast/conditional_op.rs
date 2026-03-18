use crate::{
    ast::{
        GeneratorOutputType, generate_ast, parse_children_vec,
        types::{ASTBlockType, ConditionalOperator},
    },
    errors::SpannedError,
    tokens_parser::types::UVParseNode,
    types::{Positional, Spanned},
};

/// Parse conditional operator declaration
pub fn parse_conditional_op(node: &UVParseNode) -> GeneratorOutputType {
    let extra = node.search_extra_children(vec!["test", "then", "else"]);

    if !extra.is_empty() {
        let first_extra = extra.first().ok_or(SpannedError::new(
            "[INTERNAL ERROR] Cannot get inner extra tag",
            node.span,
        ))?;

        return Err(SpannedError::new(
            "Found extra children inside conditional operator",
            first_extra.get_span(),
        ));
    }

    let test = match node.get_one_tag_by_name("test") {
        Some(t) if t.self_closing => Err(SpannedError::new(
            "`test` tag could not be self-closing",
            t.span,
        )),
        Some(t) if t.children_len() != 1 || !t.all_tags() => Err(SpannedError::new_tipped(
            "`test` should have only one nested tag",
            "If you want to place multiple tags inside, use the <g> grouping block.",
            t.span,
        )),

        Some(t) => generate_ast(t.get_tag_at(0).ok_or(SpannedError::new(
            "[INTERNAL ERROR] Cannot get inner tag",
            node.span,
        ))?),

        None => Err(SpannedError::new(
            "Conditional operator must have an `test` block inside",
            node.span,
        )),
    }?;

    Ok(ASTBlockType::ConditionalOp(Box::new(ConditionalOperator {
        test,
        then_body: parse_outcomes(node, "then")?,
        else_body: parse_outcomes(node, "else")?,
        span: node.span,
    })))
}

/// Parse conditional operator outcomes
fn parse_outcomes(
    node: &UVParseNode,
    tag_name: impl Into<String>,
) -> Result<Option<Spanned<Vec<ASTBlockType>>>, SpannedError> {
    match node.get_one_tag_by_name(tag_name.into().as_str()) {
        Some(t) if t.self_closing => Err(SpannedError::new(
            "`then` tag could not be self-closing",
            t.span,
        )),
        Some(t) if !t.all_tags() => {
            let extra_lit = t.get_inner_literal().ok_or(SpannedError::new(
                "[INTERNAL ERROR] Cannot get inner literal for error",
                t.span,
            ))?;

            Err(SpannedError::new(
                "Found unexpected literal",
                extra_lit.span,
            ))
        }
        Some(t) => Ok(Some(Spanned::new(parse_children_vec(t)?, t.span))),
        None => Ok(None),
    }
}
