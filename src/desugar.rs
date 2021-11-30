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
        match condition {
            MatchScrutinee::CatchAll => unimplemented!(),
            MatchScrutinee::Scrutinee(scrutinee) => {
                let matches = matcher(&primary, scrutinee, namespace);
                match matches {
                    Some((match_req_map, match_impl_map)) => {
                        matched_branches.push((result.to_owned(), match_req_map, match_impl_map))
                    }
                    None => return Err("Incompatible match provided".to_string()),
                }
            }
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
                if_statement = Some(Node::IfExpression(IfExpression {
                    primary: conditional.unwrap(),
                    left: Expression::CodeBlock {
                        contents: CodeBlock {
                            contents: code_block_stmts,
                        },
                    },
                    right: Some(Expression::CodeBlock {
                        contents: CodeBlock {
                            contents: the_contents,
                        },
                    }),
                }));
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
            woah => {
                println!("{:?}", woah);
                unimplemented!()
            }
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
    fn match_u32() {
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
}
