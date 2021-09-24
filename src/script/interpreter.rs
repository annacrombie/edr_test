use std::collections::HashMap;

use crate::error::Error;
use crate::script::{AstNode, AstNodeType};

type Scope = HashMap<String, String>;

pub fn interp_expr(expr: &AstNode, scope: &Scope) -> Result<String, Error> {
    match expr.t {
        AstNodeType::Call(func, ref args_node) => {
            let mut args: Vec<String> = vec![];

            for arg_node in args_node.iter() {
                args.push(interp_expr(arg_node, scope)?);
            }

            match func(&args) {
                Ok(s) => Ok(s),
                Err(s) => Err(Error {
                    loc: expr.loc,
                    msg: s,
                }),
            }
        }
        AstNodeType::Str(ref s) => Ok(s.into()),
        AstNodeType::Id(ref id) => {
            if let Some(s) = scope.get(id) {
                Ok(s.into())
            } else {
                Err(Error {
                    loc: expr.loc,
                    msg: "undefined variable".into(),
                })
            }
        }
        _ => {
            unreachable!();
        }
    }
}

pub fn interp(ast: &[AstNode]) -> Result<(), Error> {
    let mut scope: Scope = HashMap::new();

    for stmt in ast {
        match stmt.t {
            AstNodeType::Assignment(ref id, ref expr) => {
                let res = interp_expr(&expr[0], &scope)?;
                scope.insert(id.into(), res);
            }
            _ => {
                interp_expr(stmt, &scope)?;
            }
        }
    }

    Ok(())
}
