use std::collections::HashMap;

pub type Namespace<'sc> = HashMap<String, Expression<'sc>>;

#[derive(Debug)]
pub struct Tree<'sc> {
    pub nodes: Vec<Node<'sc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node<'sc> {
    Declaration(Declaration<'sc>),
    Expression(Expression<'sc>),
    WhileLoop(WhileLoop<'sc>),
    ReturnStatement(ReturnStatement<'sc>),
    MatchStatement(MatchStatement<'sc>),
    IfExpression(IfExpression<'sc>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration<'sc> {
    VariableDeclaration(VariableDeclaration<'sc>),
    Reassignment(Reassignment<'sc>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'sc> {
    Literal {
        value: Literal<'sc>,
    },
    VariableExpression {
        name: Ident<'sc>,
    },
    BinOp {
        op2: Op2,
        left: Box<Expression<'sc>>,
        right: Box<Expression<'sc>>,
    },
    Unit {},
    Array {
        contents: Vec<Expression<'sc>>,
    },
    CodeBlock {
        contents: CodeBlock<'sc>,
    },
    IfExp {
        condition: Box<Expression<'sc>>,
        then: Box<Expression<'sc>>,
        r#else: Option<Box<Expression<'sc>>>,
    },
    Tuple {
        elems: Vec<Expression<'sc>>,
    },
    StructExpression {
        struct_name: Ident<'sc>,
        fields: Vec<StructExpressionField<'sc>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op2 {
    And,
    Eq,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop<'sc> {
    pub condition: Expression<'sc>,
    pub body: CodeBlock<'sc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement<'sc> {
    pub expr: Expression<'sc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock<'sc> {
    pub contents: Vec<Node<'sc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration<'sc> {
    pub name: Ident<'sc>,
    pub body: Expression<'sc>,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ident<'sc> {
    pub primary_name: &'sc str,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct StructExpressionField<'sc> {
    pub name: Ident<'sc>,
    pub value: Expression<'sc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchStatement<'sc> {
    pub primary: Expression<'sc>,
    pub branches: Vec<MatchBranch<'sc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpression<'sc> {
    pub primary: Expression<'sc>,
    pub left: Expression<'sc>,
    pub right: Option<Expression<'sc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchBranch<'sc> {
    pub condition: MatchScrutinee<'sc>,
    pub result: Expression<'sc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchScrutinee<'sc> {
    CatchAll,
    Scrutinee(Scrutinee<'sc>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Scrutinee<'sc> {
    Literal {
        value: Literal<'sc>,
    },
    VariableExpression {
        name: Ident<'sc>,
    },
    Tuple {
        elems: Vec<Scrutinee<'sc>>,
    },
    StructScrutinee {
        struct_name: Ident<'sc>,
        fields: Vec<StructScrutineeField<'sc>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructScrutineeField<'sc> {
    pub scrutinee: Scrutinee<'sc>,
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

    pub fn expression<'sc>(exp: Expression<'sc>) -> Node <'sc> {
        Node::Expression(exp)
    }

    pub fn return_<'sc>(expr: Expression<'sc>) -> Node<'sc> {
        Node::ReturnStatement(ReturnStatement { expr })
    }

    pub fn reassignment<'sc>(lhs: Expression<'sc>, rhs: Expression<'sc>) -> Node<'sc> {
        Node::Declaration(Declaration::Reassignment(Reassignment {
            lhs: Box::new(lhs),
            rhs,
        }))
    }

    pub fn match_<'sc>(primary: Expression<'sc>, branches: Vec<MatchBranch<'sc>>) -> Node<'sc> {
        Node::MatchStatement(MatchStatement { primary, branches })
    }

    pub fn match_branch<'sc>(
        condition: MatchScrutinee<'sc>,
        result: Expression<'sc>,
    ) -> MatchBranch<'sc> {
        MatchBranch { condition, result }
    }

    pub fn match_scrutinee<'sc>(scrutinee: Scrutinee<'sc>) -> MatchScrutinee<'sc> {
        MatchScrutinee::Scrutinee(scrutinee)
    }

    pub fn variable<'sc>(name: &'sc str) -> Expression<'sc> {
        Expression::VariableExpression {
            name: Ident { primary_name: name },
        }
    }

    pub fn literal<'sc>(lit: Literal<'sc>) -> Expression<'sc> {
        Expression::Literal { value: lit }
    }

    pub fn struct_<'sc>(
        name: &'sc str,
        fields: Vec<StructExpressionField<'sc>>,
    ) -> Expression<'sc> {
        Expression::StructExpression {
            struct_name: Ident { primary_name: name },
            fields,
        }
    }

    pub fn struct_field<'sc>(name: &'sc str, value: Expression<'sc>) -> StructExpressionField<'sc> {
        StructExpressionField {
            name: Ident { primary_name: name },
            value,
        }
    }

    pub fn tuple<'sc>(elems: Vec<Expression<'sc>>) -> Expression<'sc> {
        Expression::Tuple { elems }
    }

    pub fn boolean<'sc>(b: bool) -> Literal<'sc> {
        Literal::Boolean(b)
    }

    pub fn block<'sc>(nodes: Vec<Node<'sc>>) -> Expression<'sc> {
        Expression::CodeBlock {
            contents: CodeBlock { contents: nodes },
        }
    }

    pub fn binop_and<'sc>(left: Expression<'sc>, right: Expression<'sc>) -> Expression<'sc> {
        Expression::BinOp {
            op2: Op2::And,
            left: Box::new(left),
            right: Box::new(right)
        }
    }

    pub fn binop_eq<'sc>(left: Expression<'sc>, right: Expression<'sc>) -> Expression<'sc> {
        Expression::BinOp {
            op2: Op2::Eq,
            left: Box::new(left),
            right: Box::new(right)
        }
    }

    pub fn u32_<'sc>(n: u32) -> Literal<'sc> {
        Literal::U32(n)
    }

    pub fn literal_scrutinee<'sc>(lit: Literal<'sc>) -> Scrutinee<'sc> {
        Scrutinee::Literal { value: lit }
    }

    pub fn variable_scrutinee<'sc>(name: &'sc str) -> Scrutinee<'sc> {
        Scrutinee::VariableExpression {
            name: Ident { primary_name: name },
        }
    }

    pub fn tuple_scrutinee<'sc>(elems: Vec<Scrutinee<'sc>>) -> Scrutinee<'sc> {
        Scrutinee::Tuple { elems }
    }

    pub fn struct_scrutinee<'sc>(
        name: &'sc str,
        fields: Vec<StructScrutineeField<'sc>>,
    ) -> Scrutinee<'sc> {
        Scrutinee::StructScrutinee {
            struct_name: Ident { primary_name: name },
            fields,
        }
    }

    pub fn struct_scrutinee_field<'sc>(scrutinee: Scrutinee<'sc>) -> StructScrutineeField<'sc> {
        StructScrutineeField { scrutinee }
    }

    pub fn if_statement<'sc>(primary: Expression<'sc>, left: Expression<'sc>, right: Option<Expression<'sc>>) -> Node<'sc> {
        Node::IfExpression(IfExpression {
            primary,
            left,
            right
        })
    }
}
