#[cfg(test)]
mod test {
    use crate::{interpreter::interpret, language::constructors::*, transformer::transform};

    //#[test]
    fn match_boolean() {
        let tree = tree(vec![
            variable_declaraction("x", literal(boolean(true)), false),
            match_(
                variable("x"),
                vec![
                    match_branch(
                        match_scrutinee(literal_scrutinee(boolean(false))),
                        block(vec![return_(literal(boolean(false)))]),
                    ),
                    match_branch(
                        match_scrutinee(literal_scrutinee(boolean(true))),
                        block(vec![return_(literal(boolean(true)))]),
                    ),
                ],
            ),
        ]);
        let transformed_tree = transform(tree);
        let result = interpret(transformed_tree);
        assert_eq!(result, literal(boolean(true)));
    }
}
