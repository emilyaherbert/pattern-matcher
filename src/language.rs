use std::collections::HashMap;

#[derive(Debug)]
pub struct Tree<'sc> {
    pub nodes: Vec<Node<'sc>>,
}

#[derive(Debug, Clone)]
pub enum Node<'sc> {
    Declaration(Declaration<'sc>),
    Expression(Expression<'sc>),
    WhileLoop(WhileLoop<'sc>),
    ReturnStatement(ReturnStatement<'sc>),
}

#[derive(Debug, Clone)]
pub enum Declaration<'sc> {
    VariableDeclaration(VariableDeclaration<'sc>),
    Reassignment(Reassignment<'sc>),
}

#[derive(Debug, Clone)]
pub enum Expression<'sc> {
    Literal {
        value: Literal<'sc>,
    },
    VariableExpression {
        name: Ident<'sc>,
    },
    Unit {},
    Array {
        contents: Vec<Expression<'sc>>,
    },
    MatchExpression {
        primary_expression: Box<Expression<'sc>>,
        branches: Vec<MatchBranch<'sc>>,
    },
    CodeBlock {
        contents: CodeBlock<'sc>,
    },
    IfExp {
        condition: Box<Expression<'sc>>,
        then: Box<Expression<'sc>>,
        r#else: Option<Box<Expression<'sc>>>,
    },
}

#[derive(Debug, Clone)]
pub struct WhileLoop<'sc> {
    pub condition: Expression<'sc>,
    pub body: CodeBlock<'sc>,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement<'sc> {
    pub expr: Expression<'sc>,
}

#[derive(Debug, Clone)]
pub struct CodeBlock<'sc> {
    pub contents: Vec<Node<'sc>>,
    pub scope: HashMap<&'sc str, Declaration<'sc>>,
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration<'sc> {
    pub name: Ident<'sc>,
    pub body: Expression<'sc>,
    pub is_mutable: bool,
}

#[derive(Debug, Clone)]
pub struct Ident<'sc> {
    pub primary_name: &'sc str,
}

#[derive(Debug, Clone)]
pub struct Reassignment<'sc> {
    // the thing being reassigned
    pub lhs: Box<Expression<'sc>>,
    // the expression that is being assigned to the lhs
    pub rhs: Expression<'sc>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Literal<'sc> {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    String(&'sc str),
    Boolean(bool),
    Byte(u8),
    B256([u8; 32]),
}

#[derive(Debug, Clone)]
pub struct MatchBranch<'sc> {
    pub condition: MatchCondition<'sc>,
    pub result: Expression<'sc>,
}

#[derive(Debug, Clone)]
pub enum MatchCondition<'sc> {
    CatchAll,
    Expression(Expression<'sc>),
}

pub mod constructors {
    use super::super::language::*;

    pub fn tree(nodes: Vec<Node>) -> Tree {
        Tree { nodes }
    }

    pub fn variable_declaraction<'sc>(
        name: &'sc str,
        body: Expression<'sc>,
        is_mutable: bool,
    ) -> Node<'sc> {
        Node::Declaration(Declaration::VariableDeclaration(VariableDeclaration {
            name: Ident { primary_name: name },
            body,
            is_mutable,
        }))
    }

    pub fn return_<'sc>(expr: Expression<'sc>) -> Node<'sc> {
        Node::ReturnStatement(ReturnStatement {
            expr
        })
    }

    pub fn reassignment<'sc>(lhs: Expression<'sc>, rhs: Expression<'sc>) -> Node<'sc> {
        Node::Declaration(Declaration::Reassignment(Reassignment {
            lhs: Box::new(lhs),
            rhs,
        }))
    }

    pub fn variable_expression<'sc>(name: &'sc str) -> Expression<'sc> {
        Expression::VariableExpression {
            name: Ident {
                primary_name: name
            }
        }
    }

    pub fn literal<'sc>(lit: Literal<'sc>) -> Expression<'sc> {
        Expression::Literal {
            value: lit
        }
    }

    pub fn boolean<'sc>(b: bool) -> Literal<'sc> {
        Literal::Boolean(b)
    }
}
