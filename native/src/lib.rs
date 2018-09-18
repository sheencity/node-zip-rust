#[macro_use]
extern crate neon;
extern crate walkdir;
extern crate zip;

use neon::prelude::*;
use walkdir::WalkDir;
use std::io;
use std::io::{Read, Write};
use std::fs;
use std::fs::File;
use std::net::UdpSocket;
use std::path::{Path, PathBuf};

struct ExtractTask {
    filepath: String,
    dest: String,
    socket: UdpSocket,
}

impl Task for ExtractTask {
    type Output = String;
    type Error = String;
    type JsEvent = JsString;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&self.filepath);
        let file = File::open(path).unwrap();

        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut outfile: File;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();

            let outpath: PathBuf = [&self.dest, file.name()].iter().collect();
            let display = outpath.as_path().display();
            self.socket.send(format!("{}", display).as_bytes()).unwrap();

            if (&*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath).unwrap();
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p).unwrap();
                    }
                }

                outfile = File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }
        }

        Ok(String::from("done"))
    }

    fn complete<'a>(
        self,
        mut cx: TaskContext<'a>,
        result: Result<Self::Output, Self::Error>,
    ) -> JsResult<Self::JsEvent> {
        Ok(cx.string(&result.unwrap()))
    }
}

struct CompressTask {
    src: String,
    filepath: String,
    socket: UdpSocket,
}

impl Task for CompressTask {
    type Output = String;
    type Error = String;
    type JsEvent = JsString;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let file = File::create(Path::new(&self.filepath));

        let mut zip = zip::ZipWriter::new(file.unwrap());

        let options = zip::write::FileOptions::default();

        WalkDir::new(&self.src)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| entry.path().is_file())
            .map(|file_entry| {
                let path = file_entry.path();
                let filename = path.strip_prefix(Path::new(&self.src))
                    .unwrap()
                    .to_str()
                    .unwrap();

                let output = format!("{}", path.display());
                self.socket.send(&output.as_bytes()).unwrap();

                zip.start_file(filename, options).unwrap();

                let mut file = File::open(path).unwrap();
                let mut buffer = Vec::new();

                file.read_to_end(&mut buffer).unwrap();
                zip.write_all(&*buffer).unwrap()
            })
            .count();

        Ok(String::from("done"))
    }

    fn complete<'b>(
        self,
        mut cx: TaskContext<'b>,
        result: Result<Self::Output, Self::Error>,
    ) -> JsResult<Self::JsEvent> {
        Ok(cx.string(&result.unwrap()))
    }
}

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

fn extract(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let udp_port = cx.argument::<JsNumber>(0)?.value() as u32;
    let filepath = cx.argument::<JsString>(1)?.value();
    let dest = cx.argument::<JsString>(2)?.value();
    let callback = cx.argument::<JsFunction>(3)?;

    let socket = UdpSocket::bind(format!("127.0.0.1:{}", udp_port + 1)).unwrap();
    socket.set_nonblocking(true).unwrap();
    socket.connect(format!("127.0.0.1:{}", udp_port)).unwrap();

    let task = ExtractTask {
        filepath: filepath.clone(),
        socket: socket,
        dest,
    };

    let path = Path::new(&filepath);
    let file = File::open(path).unwrap();
    let archive = zip::ZipArchive::new(file).unwrap();
    let total = cx.number(archive.len() as f64);

    task.schedule(callback);

    Ok(total)
}

fn compress(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let udp_port = cx.argument::<JsNumber>(0)?.value() as u32;
    let src = cx.argument::<JsString>(1)?.value();
    let filepath = cx.argument::<JsString>(2)?.value();
    let callback = cx.argument::<JsFunction>(3)?;

    let socket = UdpSocket::bind(format!("127.0.0.1:{}", udp_port + 1)).unwrap();
    socket.set_nonblocking(true).unwrap();
    socket.connect(format!("127.0.0.1:{}", udp_port)).unwrap();

    let total = cx.number(
        WalkDir::new(&src)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| entry.path().is_file())
            .count() as f64,
    );

    let task = CompressTask {
        src,
        filepath,
        socket,
    };

    task.schedule(callback);

    Ok(total)
}

register_module!(mut cx, {
    cx.export_function("hello", hello)?;
    cx.export_function("extract", extract)?;
    cx.export_function("compress", compress)?;
    Ok(())
});
