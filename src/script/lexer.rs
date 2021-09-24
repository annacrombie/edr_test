use crate::error::{error, Error, SrcLoc};

#[derive(Debug, Clone)]
pub enum TokenType {
    Id(String),
    Str(String),
    Eql,
    Eol,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub loc: SrcLoc,
    pub t: TokenType,
}

fn lex_str(loc: SrcLoc, chars: std::str::Chars, consumed: &mut u32) -> Result<Token, Error> {
    let mut s: String = "".into();

    let mut found_quote = 0;

    for c in chars {
        *consumed += 1;

        if c == '"' {
            found_quote += 1;
            if found_quote == 2 {
                break;
            }
        } else {
            s.push(c);
        }
    }

    if found_quote != 2 {
        return error(loc, "missing closing quotation mark");
    }

    Ok(Token {
        loc,
        t: TokenType::Str(s),
    })
}

fn lex_id(chars: std::str::Chars, consumed: &mut u32) -> String {
    let mut s: String = "".into();

    for c in chars {
        if !matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '.') {
            break;
        } else {
            *consumed += 1;

            s.push(c);
        }
    }

    s
}

pub fn lex<R: std::io::Read>(reader: std::io::BufReader<R>) -> Result<Vec<Token>, Error> {
    use std::io::BufRead;

    let mut toks: Vec<Token> = vec![];

    let mut loc = SrcLoc { col: 0, line: 0 };

    for l in reader.lines() {
        let line = l.unwrap();
        let chars = line.chars();
        let mut consumed: u32 = 0;

        for (i, c) in chars.enumerate() {
            loc.col += 1;

            if consumed > 0 {
                consumed -= 1;
                continue;
            }

            let res = match c {
                ' ' | '\t' => None,
                '#' => {
                    consumed = line[i..].len() as u32;
                    None
                }
                '"' => Some(lex_str(loc, line[i..].chars(), &mut consumed)),
                ':' => {
                    consumed += 1;

                    let s = lex_id(line[i + 1..].chars(), &mut consumed);
                    Some(Ok(Token {
                        loc,
                        t: TokenType::Str(s),
                    }))
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let s = lex_id(line[i..].chars(), &mut consumed);
                    Some(Ok(Token {
                        loc,
                        t: TokenType::Id(s),
                    }))
                }
                '=' => Some(Ok(Token {
                    loc,
                    t: TokenType::Eql,
                })),
                _ => Some(error(loc, "invalid character")),
            };

            if consumed > 0 {
                consumed -= 1;
            }

            if let Some(res) = res {
                toks.push(res?);
            }
        }

        toks.push(Token {
            loc,
            t: TokenType::Eol,
        });

        loc.line += 1;
        loc.col = 0;
    }

    toks.push(Token {
        loc,
        t: TokenType::Eof,
    });

    Ok(toks)
}
