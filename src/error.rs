#[derive(Clone, Copy, Debug)]
pub struct SrcLoc {
    pub line: u32,
    pub col: u32,
}

#[derive(Debug)]
pub struct Error {
    pub loc: SrcLoc,
    pub msg: String,
}

fn get_line(path: &str, line: usize) -> Option<String> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let reader = BufReader::new(File::open(path).expect("Cannot open file"));

    for (i, l) in reader.lines().enumerate() {
        if i == line {
            return Some(l.unwrap());
        }
    }

    None
}

impl Error {
    pub fn display(&self, src: &str, script_is_source: bool) {
        println!("error: {}", self.msg);

        let (path, line) = if script_is_source {
            ("<cmd>", Some(src.into()))
        } else {
            (src, get_line(src, self.loc.line as usize))
        };

        let pre = format!("{}:{}:{} | ", path, self.loc.line + 1, self.loc.col);
        println!("{}{}", pre, line.unwrap_or_else(|| "???".into()));
        println!("{:>width$}", "^", width = self.loc.col as usize + pre.len());
    }
}

pub fn error<T>(loc: SrcLoc, msg: &str) -> Result<T, Error> {
    Err(Error {
        loc,
        msg: msg.into(),
    })
}
