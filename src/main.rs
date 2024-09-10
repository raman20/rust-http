use std::{
    fs, io::Write, net::{TcpListener, TcpStream}, thread, time::Duration
};

use app::Request;
use app::ThreadPool;

const ADDR: &str = "127.0.0.1:7990";

fn main() {
    let listener: TcpListener = TcpListener::bind(ADDR).unwrap();
    let thread_pool: ThreadPool = ThreadPool::new(10);

    println!("started listning on addr http://{}", ADDR);

    listener
        .incoming()
        .for_each(|stream: Result<TcpStream, std::io::Error>| {
            let stream = stream.unwrap();
            thread_pool.execute(|| handle_connection(stream));
        });
}

fn handle_connection(mut stream: TcpStream){
    let request = Request::new(stream.try_clone().unwrap()).unwrap();

    let (status_line, filename) = if request.path == "/" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if request.path == "/sleep" {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let content = fs::read_to_string(filename).unwrap();
    let length = content.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");
    stream.write_all(response.as_bytes()).unwrap();
}
