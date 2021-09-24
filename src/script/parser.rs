use crate::error::{error, Error, SrcLoc};
use crate::registry::{ActivityFunc, Registry};
use crate::script::lexer::{Token, TokenType};

pub enum AstNodeType {
    Assignment(String, Vec<AstNode>),
    Call(ActivityFunc, Vec<AstNode>),
    Str(String),
    Id(String),
    Empty,
}

pub struct AstNode {
    pub loc: SrcLoc,
    pub t: AstNodeType,
}

fn get_token(toks: &[Token], i: usize) -> Result<&Token, Error> {
    match toks.get(i) {
        Some(v) => Ok(v),
        None => error(toks.last().unwrap().loc, "unexpected eof"),
    }
}

fn parse_value(toks: &[Token], mut cur: usize) -> Result<(AstNode, usize), Error> {
    let c = get_token(toks, cur)?;
    let t = match c.t {
        TokenType::Id(ref id) => {
            cur += 1;
            AstNodeType::Id(id.into())
        }
        TokenType::Str(ref s) => {
            cur += 1;
            AstNodeType::Str(s.into())
        }
        _ => AstNodeType::Empty,
    };

    Ok((AstNode { loc: c.loc, t }, cur))
}

fn parse_args(toks: &[Token], mut cur: usize) -> Result<(Vec<AstNode>, usize), Error> {
    let mut args: Vec<AstNode> = vec![];

    loop {
        let (val, ncur) = parse_value(toks, cur)?;
        cur = ncur;

        if matches!(val.t, AstNodeType::Empty) {
            break;
        }

        args.push(val);
    }

    Ok((args, cur))
}

fn parse_call(
    toks: &[Token],
    mut cur: usize,
    id: &str,
    loc: SrcLoc,
    registry: &Registry,
) -> Result<(AstNode, usize), Error> {
    let (args, ncur) = parse_args(toks, cur)?;
    cur = ncur;
    let func = registry.lookup_with_err(loc, id, args.len())?;

    Ok((
        AstNode {
            loc,
            t: AstNodeType::Call(func, args),
        },
        cur,
    ))
}

fn parse_stmt(
    toks: &[Token],
    mut cur: usize,
    registry: &Registry,
) -> Result<(AstNode, usize), Error> {
    let c = get_token(toks, cur)?;

    match c.t {
        TokenType::Id(ref id) => {
            cur += 1;
            let next = get_token(toks, cur)?;

            match next.t {
                TokenType::Eql => {
                    cur += 1;

                    let (rhs, ncur) = {
                        let c = get_token(toks, cur)?;
                        match c.t {
                            TokenType::Id(ref id) => parse_call(toks, cur + 1, id, c.loc, registry),
                            _ => parse_value(toks, cur),
                        }
                    }?;

                    cur = ncur;

                    Ok((
                        AstNode {
                            loc: c.loc,
                            t: AstNodeType::Assignment(id.into(), vec![rhs]),
                        },
                        cur,
                    ))
                }
                _ => parse_call(toks, cur, id, c.loc, registry),
            }
        }
        _ => error(c.loc, &format!("unexpected token {:?} in statement", c.t)),
    }
}

pub fn parse(tokens: Vec<Token>, registry: &Registry) -> Result<Vec<AstNode>, Error> {
    let mut stmts: Vec<AstNode> = vec![];
    let mut cur = 0;

    loop {
        let next = get_token(&tokens, cur)?;

        match next.t {
            TokenType::Eof => {
                break;
            }
            TokenType::Eol => {
                cur += 1;
                continue;
            }
            _ => (),
        }

        let (stmt, ncur) = parse_stmt(&tokens, cur, registry)?;
        cur = ncur;

        let next = get_token(&tokens, cur)?;

        if !matches!(next.t, TokenType::Eol) {
            return error(next.loc, &format!("expected end of line, got {:?}", next.t));
        }

        cur += 1;

        stmts.push(stmt);
    }

    Ok(stmts)
}
