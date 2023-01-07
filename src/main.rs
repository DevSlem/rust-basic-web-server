use std::{
    fs,
    net::{TcpListener, TcpStream}, 
    io::{prelude::*, BufReader},
    thread,
    time::Duration,
};

use rust_basic_web_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    // server가 안전하게 종료되는지 테스트하기 위해 10개의 request만 처리
    for stream in listener.incoming().take(10) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    // BufReader는 std::io::BufRead trait을 구현 -> lines() method 제공
    let buf_reader = BufReader::new(&mut stream);

    // 첫 번째 줄: HTTP의 request line
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        // 기본 페이지
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        // 처리 시간이 오래 걸리는 페이지
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        // 404 error
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    // HTML file 불러오기
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    // response 보내기
    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    );
    stream.write_all(response.as_bytes()).unwrap();
}