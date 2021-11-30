use crate::language::*;

// if (x == y)
pub type MatchReqMap<'sc> = Vec<(Expression<'sc>, Expression<'sc>)>;
// let z = 4;
pub type MatchImplMap<'sc> = Vec<(&'sc str, Expression<'sc>)>;

pub fn matcher<'sc>(
    exp: &Expression<'sc>,
    scrutinee: &Scrutinee<'sc>,
    namespace: &Namespace<'sc>,
) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    let exp = eval_exp(exp, namespace);
    match scrutinee {
        Scrutinee::Literal { value: n } => match_literal(&exp, n),
        Scrutinee::VariableExpression { name } => {
            let match_req_map = vec![];
            let match_impl_map = vec![(name.primary_name, exp.clone())];
            Some((match_req_map, match_impl_map))
        }
        Scrutinee::Tuple { elems } => match_tuple(&exp, elems, namespace),
        Scrutinee::StructScrutinee {
            struct_name,
            fields,
        } => match_struct(&exp, struct_name, fields, namespace),
        x => unimplemented!(),
    }
}

fn match_literal<'sc>(
    exp: &Expression<'sc>,
    n: &Literal<'sc>,
) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    match exp {
        Expression::Literal { value: m } => {
            if n == m {
                let match_req_map = vec![(
                    Expression::Literal { value: n.clone() },
                    Expression::Literal { value: m.clone() },
                )];
                let match_impl_map = vec![];
                Some((match_req_map, match_impl_map))
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
) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    match exp {
        Expression::Tuple { elems } => {
            if elems.len() != scrutinee_elems.len() {
                return None;
            }
            let mut match_req_maps = vec![];
            let mut match_impl_maps = vec![];
            for (elem, scrutinee_elem) in elems.iter().zip(scrutinee_elems.iter()) {
                match matcher(elem, scrutinee_elem, namespace) {
                    Some((mut match_req_map, mut match_impl_map)) => {
                        match_req_maps.append(&mut match_req_map);
                        match_impl_maps.append(&mut match_impl_map);
                    }
                    None => return None,
                }
            }
            Some((match_req_maps, match_impl_maps))
        }
        _ => None,
    }
}

fn match_struct<'sc>(
    exp: &Expression<'sc>,
    scrutinee_struct_name: &Ident<'sc>,
    scrutinee_fields: &Vec<StructScrutineeField<'sc>>,
    namespace: &Namespace<'sc>,
) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    match exp {
        Expression::StructExpression {
            struct_name,
            fields,
        } => {
            if struct_name.primary_name != scrutinee_struct_name.primary_name {
                return None;
            }
            let mut match_req_maps = vec![];
            let mut match_impl_maps = vec![];
            for (field, scrutinee_field) in fields.iter().zip(scrutinee_fields.iter()) {
                let scrutinee = scrutinee_field.scrutinee.clone();
                match scrutinee {
                    // if the scrutinee is simply naming the struct field ...
                    Scrutinee::VariableExpression { name } => {
                        if field.name.primary_name != name.primary_name {
                            return None;
                        }
                        match_impl_maps
                            .push((name.primary_name, eval_exp(&field.value, namespace)));
                    }
                    // or if the scrutinee has a more complex agenda
                    scrutinee => match matcher(&field.value, &scrutinee.clone(), namespace) {
                        Some((mut match_req_map, mut match_impl_map)) => {
                            match_req_maps.append(&mut match_req_map);
                            match_impl_maps.append(&mut match_impl_map);
                        }
                        None => return None,
                    },
                }
            }
            Some((match_req_maps, match_impl_maps))
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
        Expression::Tuple { elems } => Expression::Tuple {
            elems: elems.clone(),
        },
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
        let (match_req_map, match_impl_map) = matches.unwrap();
        assert!(match_impl_map.is_empty());
        assert_eq!(match_req_map.len(), 1);
    }

    #[test]
    fn match_variable() {
        let mut namespace = HashMap::new();
        namespace.insert("x".to_string(), literal(u32_(4)));
        let exp = variable("x");
        let scrutinee = literal_scrutinee(u32_(4));
        let matches = matcher(&exp, &scrutinee, &namespace);
        let (match_req_map, match_impl_map) = matches.unwrap();
        assert!(match_impl_map.is_empty());
        assert_eq!(match_req_map.len(), 1);
    }

    #[test]
    fn u32_to_variable() {
        let namespace = HashMap::new();
        let exp = literal(u32_(4));
        let scrutinee = variable_scrutinee("x");
        let matches = matcher(&exp, &scrutinee, &namespace);
        let (match_req_map, match_impl_map) = matches.unwrap();
        assert_eq!(match_impl_map.len(), 1);
        assert_eq!(match_req_map.len(), 0);
    }

    #[test]
    fn variable_to_variable() {
        let mut namespace = HashMap::new();
        namespace.insert("x".to_string(), literal(u32_(4)));
        let exp = variable("x");
        let scrutinee = variable_scrutinee("y");
        let matches = matcher(&exp, &scrutinee, &namespace);
        let (match_req_map, match_impl_map) = matches.unwrap();
        assert_eq!(match_impl_map.len(), 1);
        assert_eq!(match_req_map.len(), 0);
    }

    #[test]
    fn tuple_to_variable() {
        let namespace = HashMap::new();
        let exp = tuple(vec![literal(u32_(2)), literal(u32_(4))]);
        let scrutinee = variable_scrutinee("x");
        let matches = matcher(&exp, &scrutinee, &namespace);
        let (match_req_map, match_impl_map) = matches.unwrap();
        assert_eq!(match_impl_map.len(), 1);
        assert_eq!(match_req_map.len(), 0);
    }

    #[test]
    fn tuple_to_tuple() {
        let namespace = HashMap::new();
        let exp = tuple(vec![literal(u32_(2)), literal(u32_(4))]);
        let scrutinee =
            tuple_scrutinee(vec![literal_scrutinee(u32_(2)), literal_scrutinee(u32_(4))]);
        let matches = matcher(&exp, &scrutinee, &namespace);
        let (match_req_map, match_impl_map) = matches.unwrap();
        assert_eq!(match_impl_map.len(), 0);
        assert_eq!(match_req_map.len(), 2);
    }

    #[test]
    fn tuple_to_tuple_variable() {
        let namespace = HashMap::new();
        let exp = tuple(vec![literal(u32_(2)), literal(u32_(4))]);
        let scrutinee = tuple_scrutinee(vec![variable_scrutinee("x"), variable_scrutinee("y")]);
        let matches = matcher(&exp, &scrutinee, &namespace);
        let (match_req_map, match_impl_map) = matches.unwrap();
        assert_eq!(match_impl_map.len(), 2);
        assert_eq!(match_req_map.len(), 0);
    }

    #[test]
    fn tuple_to_tuple_variable_u32() {
        let namespace = HashMap::new();
        let exp = tuple(vec![literal(u32_(2)), literal(u32_(4))]);
        let scrutinee = tuple_scrutinee(vec![variable_scrutinee("x"), literal_scrutinee(u32_(4))]);
        let matches = matcher(&exp, &scrutinee, &namespace);
        let (match_req_map, match_impl_map) = matches.unwrap();
        assert_eq!(match_impl_map.len(), 1);
        assert_eq!(match_req_map.len(), 1);
    }

    #[test]
    fn tuple_none() {
        let namespace = HashMap::new();
        let exp = tuple(vec![literal(u32_(2))]);
        let scrutinee = tuple_scrutinee(vec![variable_scrutinee("x"), literal_scrutinee(u32_(4))]);
        let matches = matcher(&exp, &scrutinee, &namespace);
        assert_eq!(matches, None);
    }

    #[test]
    fn struct_to_variable() {
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
        let exp = variable("foo");
        let scrutinee = variable_scrutinee("bar");
        let matches = matcher(&exp, &scrutinee, &namespace);
        let (match_req_map, match_impl_map) = matches.unwrap();
        assert_eq!(match_impl_map.len(), 1);
        assert_eq!(match_req_map.len(), 0);
    }

    #[test]
    fn struct_to_struct() {
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
        let exp = variable("foo");
        let scrutinee = struct_scrutinee(
            "Point",
            vec![
                struct_scrutinee_field(variable_scrutinee("x")),
                struct_scrutinee_field(variable_scrutinee("y")),
            ],
        );
        let matches = matcher(&exp, &scrutinee, &namespace);
        let (match_req_map, match_impl_map) = matches.unwrap();
        assert_eq!(match_impl_map.len(), 2);
        assert_eq!(match_req_map.len(), 0);
    }

    #[test]
    fn struct_to_struct_variable() {
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
        let exp = variable("foo");
        let scrutinee = struct_scrutinee(
            "Point",
            vec![
                struct_scrutinee_field(variable_scrutinee("x")),
                struct_scrutinee_field(literal_scrutinee(u32_(7))),
            ],
        );
        let matches = matcher(&exp, &scrutinee, &namespace);
        let (match_req_map, match_impl_map) = matches.unwrap();
        assert_eq!(match_impl_map.len(), 1);
        assert_eq!(match_req_map.len(), 1);
    }

    #[test]
    fn struct_none() {
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
        let exp = variable("foo");
        let scrutinee = struct_scrutinee(
            "Point",
            vec![
                struct_scrutinee_field(variable_scrutinee("x")),
                struct_scrutinee_field(literal_scrutinee(u32_(8))),
            ],
        );
        let matches = matcher(&exp, &scrutinee, &namespace);
        assert_eq!(matches, None);
    }
}
