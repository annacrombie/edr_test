#[macro_use]
extern crate lazy_static;
extern crate getopts;

pub mod activity;
mod error;
mod logger;
mod registry;
mod script;

pub use registry::Registry;

pub fn registry() -> Registry {
    let mut registry = Registry::default();
    activity::register(&mut registry);

    registry
}

pub fn init(log_file: &str) {
    logger::init(log_file);
}

pub fn exec<R: std::io::Read>(src: R) -> Result<(), error::Error> {
    let registry = registry();
    script::exec(src, &registry)
}
