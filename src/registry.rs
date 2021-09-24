use crate::error::{Error, SrcLoc};

pub type ActivityFuncArgs<'a> = &'a [String];
pub type ActivityFuncResult = Result<String, String>;
pub type ActivityFunc = fn(ActivityFuncArgs) -> ActivityFuncResult;

pub struct RegistryElement {
    pub name: String,
    pub args: Vec<String>,
    argc_min: usize,
    argc_inf: bool,
    func: ActivityFunc,
}

#[derive(Default)]
pub struct Registry {
    funcs: Vec<RegistryElement>,
}

impl Registry {
    pub fn register(&mut self, name: &str, args: &[&str], func: ActivityFunc) {
        let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();

        let mut argc_min = 0;
        let mut argc_inf = false;

        for arg in args.iter() {
            if arg.starts_with('*') {
                if argc_inf {
                    panic!("multiple \"*\" argument types not allowed");
                }

                argc_inf = true;
            } else {
                argc_min += 1;
            }
        }

        self.funcs.push(RegistryElement {
            name: name.into(),
            args,
            func,
            argc_min,
            argc_inf,
        });
    }

    pub fn funcs(&self) -> &Vec<RegistryElement> {
        &self.funcs
    }

    pub fn lookup(&self, name: &str) -> Option<&RegistryElement> {
        self.funcs.iter().find(|e| name == e.name)
    }

    pub fn lookup_with_err(
        &self,
        loc: SrcLoc,
        name: &str,
        nargs: usize,
    ) -> Result<ActivityFunc, Error> {
        match self.lookup(name) {
            Some(e) => {
                if nargs < e.argc_min {
                    Err(Error {
                        loc,
                        msg: format!("missing arguments, expected {}", e.argc_min),
                    })
                } else if !e.argc_inf && nargs > e.argc_min {
                    Err(Error {
                        loc,
                        msg: format!("too many arguments, expected {}", e.argc_min),
                    })
                } else {
                    Ok(e.func)
                }
            }
            None => Err(Error {
                loc,
                msg: "function not found".into(),
            }),
        }
    }
}

pub fn ok() -> Result<String, String> {
    Ok("ok".to_string())
}
