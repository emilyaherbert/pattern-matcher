use crate::language::constructors::{boolean, literal, variable_scrutinee};
use crate::language::*;
use crate::matcher::*;

pub fn desugar<'sc>(node: Node<'sc>, namespace: &Namespace<'sc>) -> Result<Node<'sc>, String> {
    match node {
        Node::MatchStatement(MatchStatement { primary, branches }) => {
            desugar_match_statement(primary, branches, namespace)
        }
        node => Ok(node),
    }
}

fn desugar_match_statement<'sc>(
    primary: Expression<'sc>,
    branches: Vec<MatchBranch<'sc>>,
    namespace: &Namespace<'sc>,
) -> Result<Node<'sc>, String> {
    let mut matched_branches = vec![];
    for MatchBranch { condition, result } in branches.iter() {
        let matches = match condition {
            MatchScrutinee::CatchAll => Some((vec![], vec![])),
            MatchScrutinee::Scrutinee(scrutinee) => matcher(&primary, scrutinee, namespace),
        };
        match matches {
            Some((match_req_map, match_impl_map)) => {
                matched_branches.push((result.to_owned(), match_req_map, match_impl_map))
            }
            None => return Err("Incompatible match provided".to_string()),
        }
    }

    let mut if_statement = None;

    for (result, match_req_map, match_impl_map) in matched_branches.into_iter().rev() {
        let mut conditional = None;
        for (left_req, right_req) in match_req_map.iter() {
            let condition = Expression::BinOp {
                op2: Op2::Eq,
                left: Box::new(left_req.clone()),
                right: Box::new(right_req.clone()),
            };
            match conditional {
                None => {
                    conditional = Some(condition);
                }
                Some(the_conditional) => {
                    conditional = Some(Expression::BinOp {
                        op2: Op2::And,
                        left: Box::new(the_conditional),
                        right: Box::new(condition),
                    });
                }
            }
        }

        let mut code_block_stmts = vec![];
        for (left_impl, right_impl) in match_impl_map.into_iter() {
            code_block_stmts.push(Node::Declaration(Declaration::VariableDeclaration(
                VariableDeclaration {
                    name: Ident {
                        primary_name: left_impl,
                    },
                    body: right_impl.clone(),
                    is_mutable: false,
                },
            )));
        }
        code_block_stmts.push(Node::Expression(result.clone()));

        match if_statement {
            None => {
                let block = Expression::CodeBlock {
                    contents: CodeBlock {
                        contents: code_block_stmts,
                    },
                };
                if_statement = match conditional {
                    None => Some(Node::Expression(block)),
                    Some(conditional) => Some(Node::IfExpression(IfExpression {
                        primary: conditional,
                        left: block,
                        right: None,
                    })),
                };
            }
            Some(Node::Expression(Expression::CodeBlock {
                contents: CodeBlock {
                    contents: the_contents,
                },
            })) => {
                let left = Expression::CodeBlock {
                    contents: CodeBlock {
                        contents: code_block_stmts,
                    },
                };
                let right = Some(Expression::CodeBlock {
                    contents: CodeBlock {
                        contents: the_contents,
                    },
                });
                if_statement = match conditional {
                    None => Some(Node::IfExpression(IfExpression {
                        primary: literal(boolean(true)),
                        left,
                        right,
                    })),
                    Some(conditional) => Some(Node::IfExpression(IfExpression {
                        primary: conditional,
                        left,
                        right,
                    })),
                };
            }
            Some(Node::IfExpression(IfExpression {
                primary,
                left,
                right,
            })) => {
                if_statement = Some(Node::IfExpression(IfExpression {
                    primary: conditional.unwrap(),
                    left: Expression::CodeBlock {
                        contents: CodeBlock {
                            contents: code_block_stmts,
                        },
                    },
                    right: Some(Expression::IfExp {
                        condition: Box::new(primary),
                        then: Box::new(left),
                        r#else: right.map(Box::new),
                    }),
                }));
            }
            _ => unimplemented!(),
        }
    }

    match if_statement {
        None => Err("something went wrong...".to_string()),
        Some(if_statement) => Ok(if_statement),
    }
}

#[cfg(test)]
mod test {
    use crate::{desugar::desugar, language::constructors::*};

    use std::collections::HashMap;

    #[test]
    fn match_simple() {
        let namespace = HashMap::new();
        let node = match_(
            literal(u32_(5)),
            vec![
                match_branch(
                    match_scrutinee(literal_scrutinee(u32_(5))),
                    literal(u32_(42)),
                ),
                match_branch(match_scrutinee(variable_scrutinee("foo")), variable("foo")),
            ],
        );
        let oracle_node = if_statement(
            binop_eq(literal(u32_(5)), literal(u32_(5))),
            block(vec![expression(literal(u32_(42)))]),
            Some(block(vec![
                variable_declaraction("foo", literal(u32_(5)), false),
                expression(variable("foo")),
            ])),
        );
        let desugared = desugar(node, &namespace);
        let desugared_node = desugared.unwrap();
        assert_eq!(desugared_node, oracle_node);
    }

    #[test]
    fn match_struct() {
        let mut namespace = HashMap::new();
        namespace.insert(
            "foo".to_string(),
            struct_(
                "Point",
                vec![
                    struct_field("x", literal(u32_(5))),
                    struct_field("y", literal(u32_(7))),
                ],
            ),
        );
        let node = match_(
            variable("foo"),
            vec![
                match_branch(
                    match_scrutinee(struct_scrutinee(
                        "Point",
                        vec![
                            struct_scrutinee_field(variable_scrutinee("x")),
                            struct_scrutinee_field(literal_scrutinee(u32_(7))),
                        ],
                    )),
                    variable("x"),
                ),
                match_branch(
                    match_scrutinee(struct_scrutinee(
                        "Point",
                        vec![
                            struct_scrutinee_field(variable_scrutinee("x")),
                            struct_scrutinee_field(variable_scrutinee("y")),
                        ],
                    )),
                    variable("y"),
                ),
            ],
        );
        let oracle_node = if_statement(
            binop_eq(literal(u32_(7)), literal(u32_(7))),
            block(vec![
                variable_declaraction("x", literal(u32_(5)), false),
                expression(variable("x")),
            ]),
            Some(block(vec![
                variable_declaraction("x", literal(u32_(5)), false),
                variable_declaraction("y", literal(u32_(7)), false),
                expression(variable("y")),
            ])),
        );
        let desugared = desugar(node, &namespace);
        let desugared_node = desugared.unwrap();
        assert_eq!(desugared_node, oracle_node);
    }

    #[test]
    fn match_struct_with_scrutinee() {
        let mut namespace = HashMap::new();
        namespace.insert(
            "foo".to_string(),
            struct_(
                "Point",
                vec![
                    struct_field("x", literal(u32_(5))),
                    struct_field("y", literal(u32_(7))),
                ],
            ),
        );
        let node = match_(
            variable("foo"),
            vec![
                match_branch(
                    match_scrutinee(struct_scrutinee(
                        "Point",
                        vec![
                            struct_scrutinee_field(variable_scrutinee("x")),
                            struct_scrutinee_field(literal_scrutinee(u32_(7))),
                        ],
                    )),
                    variable("x"),
                ),
                match_branch(
                    match_scrutinee(struct_scrutinee(
                        "Point",
                        vec![
                            struct_scrutinee_field(variable_scrutinee("x")),
                            struct_scrutinee_field(variable_scrutinee("y")),
                        ],
                    )),
                    variable("y"),
                ),
                match_branch(match_scrutinee_catchall(), literal(u32_(42))),
            ],
        );

        let oracle_node = if_statement(
            binop_eq(literal(u32_(7)), literal(u32_(7))),
            block(vec![
                variable_declaraction("x", literal(u32_(5)), false),
                expression(variable("x")),
            ]),
            Some(if_expression(
                literal(boolean(true)),
                block(vec![
                    variable_declaraction("x", literal(u32_(5)), false),
                    variable_declaraction("y", literal(u32_(7)), false),
                    expression(variable("y")),
                ]),
                Some(block(vec![expression(literal(u32_(42)))])),
            )),
        );
        let desugared = desugar(node, &namespace);
        let desugared_node = desugared.unwrap();
        assert_eq!(desugared_node, oracle_node);
    }

    #[test]
    fn match_struct_multiple() {
        let mut namespace = HashMap::new();
        namespace.insert(
            "foo".to_string(),
            struct_(
                "Point",
                vec![
                    struct_field("x", literal(u32_(5))),
                    struct_field("y", literal(u32_(7))),
                ],
            ),
        );
        let node = match_(
            variable("foo"),
            vec![
                match_branch(
                    match_scrutinee(struct_scrutinee(
                        "Point",
                        vec![
                            struct_scrutinee_field(variable_scrutinee("x")),
                            struct_scrutinee_field(literal_scrutinee(u32_(0))),
                        ],
                    )),
                    variable("x"),
                ),
                match_branch(
                    match_scrutinee(struct_scrutinee(
                        "Point",
                        vec![
                            struct_scrutinee_field(literal_scrutinee(u32_(0))),
                            struct_scrutinee_field(variable_scrutinee("y")),
                        ],
                    )),
                    variable("y"),
                ),
                match_branch(
                    match_scrutinee(struct_scrutinee(
                        "Point",
                        vec![
                            struct_scrutinee_field(variable_scrutinee("x")),
                            struct_scrutinee_field(variable_scrutinee("y")),
                        ],
                    )),
                    variable("y"),
                ),
            ],
        );

        let oracle_node = if_statement(
            binop_eq(literal(u32_(0)), literal(u32_(7))),
            block(vec![
                variable_declaraction("x", literal(u32_(5)), false),
                expression(variable("x")),
            ]),
            Some(if_expression(
                binop_eq(literal(u32_(0)), literal(u32_(5))),
                block(vec![
                    variable_declaraction("y", literal(u32_(7)), false),
                    expression(variable("y")),
                ]),
                Some(block(vec![
                    variable_declaraction("x", literal(u32_(5)), false),
                    variable_declaraction("y", literal(u32_(7)), false),
                    expression(variable("y")),
                ])),
            )),
        );
        let desugared = desugar(node, &namespace);
        let desugared_node = desugared.unwrap();
        assert_eq!(desugared_node, oracle_node);
    }
}
