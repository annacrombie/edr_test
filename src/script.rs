mod interpreter;
mod lexer;
mod parser;

use crate::error::Error;
use crate::registry::Registry;
pub use parser::{AstNode, AstNodeType};

pub fn exec<R: std::io::Read>(src: R, registry: &Registry) -> Result<(), Error> {
    let reader = std::io::BufReader::new(src);
    let tokens = lexer::lex(reader)?;

    if tokens.is_empty() {
        return Ok(());
    }

    let ast = match parser::parse(tokens, registry) {
        Ok(ast) => ast,
        Err(err) => {
            return Err(err);
        }
    };

    interpreter::interp(&ast)?;

    Ok(())
}
