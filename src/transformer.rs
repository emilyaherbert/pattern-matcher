use crate::language::*;

pub fn transform<'sc>(tree: Tree<'sc>) -> Tree<'sc> {
    Tree {
        nodes: transform_nodes(tree.nodes),
    }
}

pub fn transform_nodes<'sc>(nodes: Vec<Node<'sc>>) -> Vec<Node<'sc>> {
    let mut new_tree = vec![];
    for node in nodes {
        let mut new_nodes = match node {
            Node::Declaration(declaration) => transform_declaration(declaration),
            Node::Expression(expression) => transform_expression(expression),
            Node::MatchStatement(match_statement) => transform_match_statement(match_statement),
            Node::ReturnStatement(return_statement) => transform_return_statement(return_statement),
            Node::WhileLoop(while_loop) => transform_while_loop(while_loop),
        };
        new_tree.append(&mut new_nodes);
    }
    new_tree
}

fn transform_match_statement<'sc>(match_statement: MatchStatement<'sc>) -> Vec<Node<'sc>> {
    unimplemented!()
}

fn transform_declaration<'sc>(declaration: Declaration<'sc>) -> Vec<Node<'sc>> {
    vec![match declaration {
        Declaration::Reassignment(reassignment) => {
            Node::Declaration(Declaration::Reassignment(reify_reassignment(reassignment)))
        }
        Declaration::VariableDeclaration(variable_declaration) => Node::Declaration(
            Declaration::VariableDeclaration(reify_variable_declaration(variable_declaration)),
        ),
    }]
}

fn transform_expression<'sc>(expression: Expression<'sc>) -> Vec<Node<'sc>> {
    vec![Node::Expression(reify_expression(expression))]
}

fn transform_return_statement<'sc>(return_statement: ReturnStatement<'sc>) -> Vec<Node<'sc>> {
    vec![Node::ReturnStatement(ReturnStatement {
        expr: reify_expression(return_statement.expr),
    })]
}

fn transform_while_loop<'sc>(while_loop: WhileLoop<'sc>) -> Vec<Node<'sc>> {
    vec![Node::WhileLoop(WhileLoop {
        condition: reify_expression(while_loop.condition),
        body: reify_codeblock(while_loop.body),
    })]
}

fn reify_expression<'sc>(exp: Expression<'sc>) -> Expression<'sc> {
    match exp {
        Expression::Literal { value } => Expression::Literal { value },
        Expression::VariableExpression { name } => Expression::VariableExpression { name },
        Expression::Unit {} => Expression::Unit {},
        Expression::Array { contents } => Expression::Array {
            contents: contents.into_iter().map(reify_expression).collect(),
        },
        Expression::CodeBlock { contents } => Expression::CodeBlock {
            contents: reify_codeblock(contents),
        },
        Expression::IfExp {
            condition,
            then,
            r#else,
        } => Expression::IfExp {
            condition: Box::new(reify_expression(*condition)),
            then: Box::new(reify_expression(*then)),
            r#else: r#else.map(|x| Box::new(reify_expression(*x))),
        },
        x => unimplemented!(),
    }
}

fn reify_codeblock<'sc>(codeblock: CodeBlock<'sc>) -> CodeBlock<'sc> {
    CodeBlock {
        contents: transform_nodes(codeblock.contents),
    }
}

fn reify_reassignment<'sc>(reassignment: Reassignment<'sc>) -> Reassignment<'sc> {
    Reassignment {
        lhs: Box::new(reify_expression(*reassignment.lhs)),
        rhs: reify_expression(reassignment.rhs),
    }
}

fn reify_variable_declaration<'sc>(
    variable_declaration: VariableDeclaration<'sc>,
) -> VariableDeclaration<'sc> {
    VariableDeclaration {
        name: variable_declaration.name,
        body: reify_expression(variable_declaration.body),
        is_mutable: variable_declaration.is_mutable,
    }
}
