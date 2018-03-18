use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use hyper;
use futures;
use futures::future::FutureResult;
use hyper::{Get, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

static DEFAULT_HTML_INDEX: &str = r#"
<html>
    <head>
        <meta content="text/html;charset=utf-8" http-equiv="Content-Type"/>
    </head>
    <body>
        <script src='app.js'></script>
    </body>
</html>"#;

fn path_exists(path: &Path) -> bool {
    match fs::metadata(path) {
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => false,
            _ => true,
        },
        _ => true,
    }
}

fn serve_file(path: &Path) -> Response {
    if path_exists(path) {
        let contents = {
            let mut file = File::open(path).unwrap();
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();
            buf
        };
        let mut response = Response::new().with_header(ContentLength(contents.len() as u64));
        if path.as_os_str().to_str().unwrap().ends_with(".wasm") {
            response = response.with_header(CustomContentType("application/wasm".to_string()));
        }
        response.with_body(contents)
    } else {
        Response::new().with_status(StatusCode::NotFound)
    }
}

header! { (CustomContentType, "Content-Type") => [String] }
struct WebApp {
    target: String,
    app_path: PathBuf,
}

impl Service for WebApp {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        futures::future::ok(match (req.method(), req.path()) {
            (&Get, "/") => {
                let html_index = format!("./html/{}.html", self.target);
                let html_index_path = Path::new(&html_index);
                if path_exists(html_index_path) {
                    serve_file(html_index_path)
                } else {
                    Response::new()
                        .with_header(ContentLength(DEFAULT_HTML_INDEX.len() as u64))
                        .with_body(DEFAULT_HTML_INDEX)
                }
            }
            (&Get, path) => {
                let mut file_path = self.app_path.clone();
                for (i, path_part) in path.split("/").enumerate() {
                    if i > 0 {
                        file_path.push(Path::new(path_part));
                    }
                }

                serve_file(&file_path)
            }
            _ => Response::new().with_status(StatusCode::NotFound),
        })
    }
}

pub fn serve(target_name: String, mut path: PathBuf) {
    let addr = "127.0.0.1:8000".parse().unwrap();

    path.pop();
    let server = Http::new()
        .bind(&addr, move || {
            Ok(WebApp {
                target: target_name.clone(),
                app_path: path.clone(),
            })
        })
        .unwrap();
    println!(
        "wasm-build: Listening on http://{}.",
        server.local_addr().unwrap()
    );
    server.run().unwrap();
}
