use edr_test::exec;
use std::env;
use std::fs::{DirBuilder, File};
use std::io::Read;
use std::path::PathBuf;

fn tmpdir() -> PathBuf {
    let mut dir = env::temp_dir().join("edr_test_dir.1");
    let mut n = 2;

    while dir.exists() {
        dir = dir.with_extension(format!("{}", n));
        n += 1;
    }

    DirBuilder::new().create(&dir).unwrap();

    dir
}

#[test]
fn garbage_fails_to_run() {
    assert!(matches!(exec("garbage".as_bytes()), Err(_)));
}

#[test]
fn file_gets_created() {
    let dir = tmpdir();
    let f = dir.join("test");

    assert!(!f.exists());

    exec(format!("file.create :file \"{}\"", f.to_string_lossy()).as_bytes()).unwrap();

    assert!(f.is_file());

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn dir_gets_created() {
    let dir = tmpdir();
    let f = dir.join("test");

    assert!(!f.exists());

    exec(format!("file.create :dir \"{}\"", f.to_string_lossy()).as_bytes()).unwrap();

    assert!(f.is_dir());

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn file_gets_modified() {
    let dir = tmpdir();
    let f = dir.join("test");
    let msg = "hello world";

    assert!(!f.exists());

    let f_s = f.to_string_lossy();

    exec(
        format!(
            r#"file.create :file "{}"
            file.modify "{}" "{}""#,
            f_s, f_s, msg
        )
        .as_bytes(),
    )
    .unwrap();

    assert!(f.is_file());

    let mut file = File::open(f).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    assert!(contents == msg);

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn file_gets_deleted() {
    let dir = tmpdir();
    let f = dir.join("test");

    File::create(&f).unwrap();

    assert!(f.exists());

    exec(format!("file.delete \"{}\"", f.to_string_lossy()).as_bytes()).unwrap();

    assert!(!f.exists());

    std::fs::remove_dir_all(dir).unwrap();
}
