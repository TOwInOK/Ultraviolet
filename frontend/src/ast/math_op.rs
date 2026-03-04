use crate::{
    ast::{
        GeneratorOutputType, generate_ast,
        traits::StringToUVMathOp,
        types::{ASTBlockType, MathOp},
    },
    errors::SpannedError,
    tokens_parser::types::UVParseNode,
};

pub fn parse_math_op(node: &UVParseNode) -> GeneratorOutputType {
    let op_type = node
        .name
        .to_uvmath()
        .ok_or(SpannedError::new("Unknown math operation", node.span))?;

    let children = parse_arguments(node, !op_type.is_variadic())?;

    Ok(ASTBlockType::MathOp(MathOp {
        op_type,
        operands: children,
    }))
}

/// Parse arguments for math functions
pub fn parse_arguments(
    node: &UVParseNode,
    only_two: bool,
) -> Result<Vec<ASTBlockType>, SpannedError> {
    if !node.all_tags() {
        return Err(SpannedError::new(
            "Unexpected literals inside math operation",
            node.span,
        ));
    }

    if only_two && node.children_len() != 2 {
        return Err(SpannedError::new(
            format!("`{}` math operation can handle only 2 arguments", node.name),
            node.span,
        ));
    }

    node.get_all_tags()
        .into_iter()
        .map(|ch| generate_ast(ch))
        .collect::<Result<Vec<ASTBlockType>, SpannedError>>()
}
