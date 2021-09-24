extern crate serde;
extern crate whoami;

use serde::Serialize;

#[derive(Serialize)]
struct Env {
    pid: u32,
    exe: String,
    args: Vec<String>,
    user: String,
}

lazy_static! {
    #[derive(Serialize)]
    static ref LOGENV: Env = Env {
        exe: std::env::current_exe().unwrap().to_string_lossy().into(),
        pid: std::process::id(),
        user: whoami::username(),
        args: std::env::args().collect(),
    };
}

#[derive(Serialize)]
struct Record<'a, T: Serialize> {
    timestamp: u64,
    env: &'static Env,
    activity: &'a T,
}

static mut LOG_FILE: Option<std::fs::File> = None;

pub fn init(path: &str) {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)
        .unwrap();

    unsafe {
        LOG_FILE = Some(file);
    }
}

pub fn log<T: Serialize>(activity: &T) {
    use std::io::Write;
    use std::time::SystemTime;

    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let record = Record {
        timestamp: ts,
        env: &*LOGENV,
        activity,
    };

    let json = serde_json::to_string(&record).unwrap();

    unsafe {
        if let Some(ref mut f) = LOG_FILE {
            f.write_all(json.as_bytes()).unwrap();
            f.write_all(b"\n").unwrap();
        };
    };
}
