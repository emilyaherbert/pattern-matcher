use crate::language::*;

use std::collections::HashMap;

type Namespace<'sc> = HashMap<String, Expression<'sc>>;
type MatchMap<'sc> = Vec<(String, Expression<'sc>)>;

pub fn matcher<'sc>(
    exp: &Expression<'sc>,
    scrutinee: &Scrutinee<'sc>,
    namespace: &Namespace<'sc>,
) -> Option<MatchMap<'sc>> {
    let exp = eval_exp(exp, namespace);
    match scrutinee {
        Scrutinee::Literal { value: n } => match_literal(&exp, n),
        Scrutinee::VariableExpression { name } => {
            Some(vec![(name.primary_name.to_string(), exp.clone())])
        }
        Scrutinee::Tuple { elems } => match_tuple(&exp, elems, namespace),
        x => unimplemented!(),
    }
}

fn match_literal<'sc>(exp: &Expression<'sc>, n: &Literal<'sc>) -> Option<MatchMap<'sc>> {
    match exp {
        Expression::Literal { value: m } => {
            if n == m {
                Some(vec![])
            } else {
                None
            }
        }
        _ => None,
    }
}

fn match_tuple<'sc>(
    exp: &Expression<'sc>,
    scrutinee_elems: &Vec<Scrutinee<'sc>>,
    namespace: &Namespace<'sc>,
) -> Option<MatchMap<'sc>> {
    match exp {
        Expression::Tuple { elems } => {
            if elems.len() != scrutinee_elems.len() {
                return None;
            }
            let it = elems.iter().zip(scrutinee_elems.iter());
            let mut match_map = vec![];
            for (elem, scrutinee_elem) in it {
                match matcher(elem, scrutinee_elem, namespace) {
                    Some(mut new_match_map) => match_map.append(&mut new_match_map),
                    None => return None,
                }
            }
            Some(match_map)
        }
        _ => None,
    }
}

fn eval_exp<'sc>(exp: &Expression<'sc>, namespace: &Namespace<'sc>) -> Expression<'sc> {
    match exp {
        Expression::Literal { value } => Expression::Literal {
            value: value.clone(),
        },
        Expression::VariableExpression { name } => {
            namespace.get(name.primary_name).unwrap().clone()
        }
        Expression::Tuple { elems } => Expression::Tuple { elems: elems.clone() },
        x => unimplemented!(),
    }
}

#[cfg(test)]
mod test {
    use crate::{language::constructors::*, matcher::matcher};

    use std::collections::HashMap;

    #[test]
    fn match_u32() {
        let namespace = HashMap::new();
        let exp = literal(u32_(4));
        let scrutinee = literal_scrutinee(u32_(4));
        let matches = matcher(&exp, &scrutinee, &namespace);
        assert!(matches.unwrap().is_empty());
    }

    #[test]
    fn match_variable() {
        let mut namespace = HashMap::new();
        namespace.insert("x".to_string(), literal(u32_(4)));
        let exp = variable("x");
        let scrutinee = literal_scrutinee(u32_(4));
        let matches = matcher(&exp, &scrutinee, &namespace);
        assert!(matches.unwrap().is_empty());
    }

    #[test]
    fn u32_to_variable() {
        let namespace = HashMap::new();
        let exp = literal(u32_(4));
        let scrutinee = variable_scrutinee("x");
        let matches = matcher(&exp, &scrutinee, &namespace);
        assert_eq!(matches.unwrap().len(), 1);
    }

    #[test]
    fn variable_to_variable() {
        let mut namespace = HashMap::new();
        namespace.insert("x".to_string(), literal(u32_(4)));
        let exp = variable("x");
        let scrutinee = variable_scrutinee("x");
        let matches = matcher(&exp, &scrutinee, &namespace);
        assert_eq!(matches.unwrap().len(), 1);
    }

    #[test]
    fn tuple_to_variable() {
        let namespace = HashMap::new();
        let exp = tuple(vec![literal(u32_(2)), literal(u32_(4))]);
        let scrutinee = variable_scrutinee("x");
        let matches = matcher(&exp, &scrutinee, &namespace);
        assert_eq!(matches.unwrap().len(), 1);
    }

    #[test]
    fn tuple_to_tuple() {
        let namespace = HashMap::new();
        let exp = tuple(vec![literal(u32_(2)), literal(u32_(4))]);
        let scrutinee =
            tuple_scrutinee(vec![literal_scrutinee(u32_(2)), literal_scrutinee(u32_(4))]);
        let matches = matcher(&exp, &scrutinee, &namespace);
        assert_eq!(matches.unwrap().len(), 0);
    }

    #[test]
    fn tuple_to_tuple_variable() {
        let namespace = HashMap::new();
        let exp = tuple(vec![literal(u32_(2)), literal(u32_(4))]);
        let scrutinee =
            tuple_scrutinee(vec![variable_scrutinee("x"), variable_scrutinee("y")]);
        let matches = matcher(&exp, &scrutinee, &namespace);
        assert_eq!(matches.unwrap().len(), 2);
    }

    #[test]
    fn tuple_to_tuple_variable_u32() {
        let namespace = HashMap::new();
        let exp = tuple(vec![literal(u32_(2)), literal(u32_(4))]);
        let scrutinee =
            tuple_scrutinee(vec![variable_scrutinee("x"), literal_scrutinee(u32_(4))]);
        let matches = matcher(&exp, &scrutinee, &namespace);
        assert_eq!(matches.unwrap().len(), 1);
    }
}
