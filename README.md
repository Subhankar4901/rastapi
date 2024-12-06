# RastAPI

**RastAPI** is an easy-to-use Rust library for creating low-boilerplate, efficient RESTful APIs, inspired by FastAPI (Python). Define route handlers (functions) and map them to their corresponding URLs to get a fully functional API.

RastAPI is designed with **Non-Blocking I/O** and a **Threadpool** architecture for efficient and concurrent request handling. It also features an advanced **LFU-LRU-based file caching system**, leveraging multithreading with lock-stripping techniques to minimize contention.

---

## Features

- Minimal boilerplate for defining routes and handlers.
- High performance leveraging Rust’s concurrency and safety.
- Efficient file uploads and downloads.

---

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
rastapi = "0.1.0" # Replace with the latest version
```
## Usage

### Text or JSON content
Define routes and handlers to serve text or JSON content using `create_response` function.
```rust
use rastapi::RastAPI;
use rastapi::Request::HttpRequest;
use rastapi::Response::{HttpResponse, create_response};
use rastapi::utils::ContentType;
use std::collections::HashMap;

// Define a route handler function.
// Signature: fn(request: &HttpRequest, path_params: HashMap<String, String>) -> HttpResponse
fn json(request_obj: &HttpRequest, path_params: HashMap<String, String>) -> HttpResponse {
    let json_content = r#"{
        "name": "Rony",
        "batch": 2024,
        "sem": 3,
        "subjects": ["OS", "Networking", "Algorithms"],
        "grade": {
            "OS": "C",
            "Cryptography": "C",
            "Algorithms": "C"
        }
    }"#;

    let resp = create_response(&json_content, 200, ContentType::JSON, true).unwrap();
    resp
}

fn main() {
    let mut app = RastAPI::new();
    app.set_total_workers(5); // Set the number of threads in the threadpool.
    app.register_route("/json", vec!["GET"], json); // Register the route handler.
    app.run("0.0.0.0", 5000); // Start the server.
}
```
### File Content
Similarly you can serve file content using `send_file`. If a file is not found, the server automatically responds with `404 Not Found`.
```rust
use rastapi::RastAPI;
use rastapi::Request::HttpRequest;
use rastapi::Response::{HttpResponse, send_file};
use rastapi::utils::FileType;
use std::collections::HashMap;

fn file(req: &HttpRequest, path_params: HashMap<String, String>) -> HttpResponse {
    let mut resp = send_file(
        "file_path/file_name.ext",
        Some("file_name.ext".to_string()),
        FileType::MP4,
        200,
        true,
    )
    .unwrap();

    resp.add_header("Header-Name", "Header-Value");
    resp
}

fn main() {
    let mut app = RastAPI::new();
    app.set_total_workers(5);
    app.register_route("/file", vec!["GET"], file);
    app.run("127.0.0.1", 5000);
}
```
## Future Work
- [ ] Don't avoid the rust borrow checker and take it head on. For example using `&str` instead of `String`.
- [ ] Building a more robust logging system.
- [ ] Enable encrypted connections (https). Currently it only supports http.
- [ ] Impliment security fetures like DDOS protection.
- [ ] Building a templating engiene for server side rendering of HTML.
- [ ] Enable support for http2. (maybe)
