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

    for (result, match_req_map, match_impl_map) in matched_branches.iter() {}

    unimplemented!()
}
