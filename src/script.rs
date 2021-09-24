mod lexer;
mod parser;

use crate::error::Error;
use crate::registry::Registry;
pub use parser::{AstNode, AstNodeType};

pub fn parse<R: std::io::Read>(
    reader: std::io::BufReader<R>,
    registry: &Registry,
) -> Result<Vec<AstNode>, Error> {
    let tokens = lexer::lex(reader)?;

    if tokens.is_empty() {
        return Ok(vec![]);
    }

    parser::parse(tokens, registry)
}
