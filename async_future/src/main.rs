#[macro_use]
extern crate lazy_static;
extern crate futures;
extern crate hyper;
extern crate hyper_staticfile;
extern crate rand;
extern crate regex;
extern crate tokio;

use futures::{future, Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper_staticfile::FileChunkStream;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use regex::Regex;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;
use tokio::fs::File;

// According to the Rust lang rule, static and const vars
// should be in SCREAMING_SNAKE_CASE
static INDEX: &[u8] = b"Images Microservice";

// This will make Regex only when needed(Lazy initialization)
lazy_static! {
    static ref DOWNLOAD_FILE: Regex = Regex::new("^/download/(?P<filename>\\w{20})?$").unwrap();
}

fn main() {
    // Create ./files dir under the root folder
    let files = Path::new("./files");
    // When you use Result<T, E>, you have to eventually consume it,
    // So we use ok() to consume it.
    fs::create_dir(files).ok();

    // Bound address as socket address type
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    // Bound server builder will serve the microservice(with given request/files) that we created
    let server = builder.serve(move || service_fn(move |req| microservice_handler(req, &files)));
    // Eliminate not-custom type errors
    let server = server.map_err(drop);
    // Run hyper server
    hyper::rt::run(server);
}

// Find all hyper type errors in body(map_error),
// 6. label them as "ErrorKind::Other" type(other(error)) which converts it to io type error.
fn other<E>(err: E) -> Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    Error::new(ErrorKind::Other, err)
}

fn microservice_handler(
    req: Request<Body>,
    files: &Path,
) -> Box<dyn Future<Item = Response<Body>, Error = std::io::Error> + Send> {
    match (req.method(), req.uri().path().to_owned().as_ref()) {
        // For root path, we only show Index message
        (&Method::GET, "/") => Box::new(future::ok(Response::new(INDEX.into()))),
        (&Method::POST, "/upload") => {
            // Make random 20 character length filename
            // WHy random? because in service, it's a bad idea to preserve the
            // actual name of the uploaded file, since there can be a same one already.
            // Remember why uuid is used for ids.
            let name: String = thread_rng().sample_iter(&Alphanumeric).take(20).collect();
            let mut filepath = files.to_path_buf();
            filepath.push(&name);
            // Instead of std::fs::File, we use tokio::fs::File,
            // so that we can use AsyncRead & AsyncWrite methods!
            // (basically tokio File is based on standard File)
            // Create a file(create_file). It might success or not(future).
            let create_file = File::create(filepath);
            let write = create_file
                // 2. If success(and_then), then consume Request to parse Body(into_body()).
                .and_then(|file| {
                    req.into_body()
                        // The body contains:
                        // 1) File Chunks(because file is big data)
                        // 2) Hyper type error(while tokio File is std error)
                        // To make errors compatible, we convert them using other().
                        .map_err(other)
                        // We will append chunks to file stream to make it whole
                        // using fold(cumulative function)
                        .fold(file, |file, chunk| {
                            // and this writes a chunk in to the file stream :D
                            tokio::io::write_all(file, chunk).map(|(file, _)| file)
                        })
                });
            // We respond for every file sent to the server with file name
            // We have to convert name into desirable type for Response::new()
            // using into(). It's so powerful and convenient!
            let body = write.map(|_| Response::new(name.into()));
            Box::new(body)
        }
        // When user access to /download/<filename>, then download image
        (&Method::GET, path) if path.starts_with("/download") => {
            // Analyze input path with DOWNLOAD_FILE Regex pattern
            if let Some(cap) = DOWNLOAD_FILE.captures(path) {
                // In that pattern, extract the "filename" part.
                let filename = cap.name("filename").unwrap().as_str();
                // Convert your filepath to buffer, which is File vector
                let mut filepath = files.to_path_buf();
                // And then push the filename at the end of file path
                filepath.push(filename);
                // Open the file with given filepath, which returns chunks
                let open_file = File::open(filepath);
                // For every chunk(not the full file, because it would be to big)
                let body = open_file.map(|file| {
                    // Wrap the chunks with FileChunkStream to make a stream
                    let chunks = FileChunkStream::new(file);
                    // Then wrap that stream to send as Http Response
                    Response::new(Body::wrap_stream(chunks))
                });
                Box::new(body)
            // If we cannot find the file from given input path,
            // Return NOT_FOUND response
            } else {
                response_with_code(StatusCode::NOT_FOUND)
            }
        }
        _ => response_with_code(StatusCode::NOT_FOUND),
    }
}

// Make a Http Response smart pointer for the given Http StatusCode
fn response_with_code(
    status_code: StatusCode,
) -> Box<dyn Future<Item = Response<Body>, Error = std::io::Error> + Send> {
    // Response::builder() can add infos like status, body in functional manner
    let res = Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap();
    // Response should be wrapped in Future type.
    Box::new(future::ok(res))
}
