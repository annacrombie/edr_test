use crate::logger;
use crate::registry::{ok, ActivityFuncArgs, ActivityFuncResult, Registry};

use serde::Serialize;

fn wrap_io_err<T>(result: Result<T, std::io::Error>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

fn wrap_io_res<T, U: Serialize>(result: Result<T, std::io::Error>, e: &U) -> ActivityFuncResult {
    result
        .map(|_| {
            logger::log(e);
            "ok".to_string()
        })
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
enum ActivityType {
    #[allow(non_camel_case_types)]
    file,
    #[allow(non_camel_case_types)]
    process,
    #[allow(non_camel_case_types)]
    network,
}

enum FileActivityType {
    Create,
    Modify,
    Delete,
}

#[derive(Serialize)]
struct FileActivity<'a> {
    #[serde(rename(serialize = "type"))]
    t: ActivityType,
    path: &'a str,
    activity_descriptor: &'a str,
}

impl FileActivity<'_> {
    fn new(path: &str, t: FileActivityType) -> FileActivity {
        FileActivity {
            path,
            activity_descriptor: match t {
                FileActivityType::Create => "create",
                FileActivityType::Modify => "modify",
                FileActivityType::Delete => "delete",
            },
            t: ActivityType::file,
        }
    }
}

#[derive(Serialize)]
struct ProcessSpawnActivity<'a> {
    #[serde(rename(serialize = "type"))]
    t: ActivityType,
    name: &'a str,
    command_line: &'a [String],
    pid: u32,
}

#[derive(Serialize)]
struct NetworkActivity<'a> {
    #[serde(rename(serialize = "type"))]
    t: ActivityType,
    src: &'a str,
    dest: &'a str,
    sent: usize,
    protocol: &'a str,
}

impl NetworkActivity<'_> {
    fn new<'a>(src: &'a str, dest: &'a str, sent: usize, protocol: &'a str) -> NetworkActivity<'a> {
        NetworkActivity {
            src,
            dest,
            sent,
            protocol,
            t: ActivityType::network,
        }
    }
}

fn file_create(args: ActivityFuncArgs) -> ActivityFuncResult {
    use std::fs;

    if let [ft, path] = args {
        match &ft[..] {
            "file" => wrap_io_res(
                fs::File::create(path),
                &FileActivity::new(path, FileActivityType::Create),
            ),
            "dir" => wrap_io_res(
                fs::create_dir(path),
                &FileActivity::new(path, FileActivityType::Create),
            ),
            _ => Err(format!("invalid file type: {}", ft)),
        }
    } else {
        unreachable!();
    }
}

fn file_modify(args: ActivityFuncArgs) -> ActivityFuncResult {
    use std::fs;

    if let [path, contents] = args {
        wrap_io_res(
            fs::write(path, contents.as_bytes()),
            &FileActivity::new(path, FileActivityType::Modify),
        )
    } else {
        unreachable!();
    }
}

fn file_delete(args: ActivityFuncArgs) -> ActivityFuncResult {
    use std::fs;

    if let [path] = args {
        let metadata = wrap_io_err(fs::metadata(path))?;
        let file_type = metadata.file_type();

        if file_type.is_dir() {
            wrap_io_res(
                fs::remove_dir(path),
                &FileActivity::new(path, FileActivityType::Delete),
            )
        } else if file_type.is_file() {
            wrap_io_res(
                fs::remove_file(path),
                &FileActivity::new(path, FileActivityType::Delete),
            )
        } else {
            Err(format!("unable to delete file type: {:?}", file_type))
        }
    } else {
        unreachable!();
    }
}

fn process_spawn(args: ActivityFuncArgs) -> ActivityFuncResult {
    use std::process::Command;

    wrap_io_err(Command::new(&args[0]).args(&args[1..]).spawn()).map(|p| {
        logger::log(&ProcessSpawnActivity {
            name: &args[0],
            command_line: args,
            pid: p.id(),
            t: ActivityType::process,
        });

        "ok".to_string()
    })
}

fn network_transmit(args: ActivityFuncArgs) -> ActivityFuncResult {
    use std::io::Write;
    use std::net::{AddrParseError, Ipv4Addr, SocketAddr, TcpStream, UdpSocket};

    if let [tt, dest, msg] = args {
        match &tt[..] {
            "tcp" => {
                let mut stream = wrap_io_err(TcpStream::connect(dest))?;
                wrap_io_res(
                    stream.write(msg.as_bytes()),
                    &NetworkActivity::new(
                        &stream.local_addr().unwrap().to_string(),
                        dest,
                        msg.as_bytes().len(),
                        tt,
                    ),
                )
            }
            "udp" => {
                let socket = wrap_io_err(UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)))?;
                let dest: SocketAddr = dest.parse().map_err(|e: AddrParseError| e.to_string())?;
                wrap_io_res(
                    socket.send_to(msg.as_bytes(), dest),
                    &NetworkActivity::new(
                        &socket.local_addr().unwrap().to_string(),
                        &dest.to_string(),
                        msg.as_bytes().len(),
                        tt,
                    ),
                )
            }
            _ => return Err(format!("invalid network protocol: {}", tt)),
        }
    } else {
        unreachable!();
    }
}

fn join(args: ActivityFuncArgs) -> ActivityFuncResult {
    Ok(args.join(""))
}

fn print(args: ActivityFuncArgs) -> ActivityFuncResult {
    for arg in args {
        println!("{}", arg);
    }

    ok()
}

pub fn register(registry: &mut Registry) {
    registry.register("file.create", &["type", "path"], file_create);
    registry.register("file.modify", &["path", "new contents"], file_modify);
    registry.register("file.delete", &["path"], file_delete);
    registry.register("process.spawn", &["cmd", "*args"], process_spawn);
    registry.register(
        "network.transmit",
        &["protocol", "dest", "message"],
        network_transmit,
    );
    registry.register("join", &["*args"], join);
    registry.register("print", &["message"], print);
}
