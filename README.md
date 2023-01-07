# The Rust Programming Language - Final Project: Building a Multithreaded Web Server

Rust 프로그래밍 언어 튜토리얼 [The Rust Programming Language](https://doc.rust-lang.org/book/title-page.html)를 공부하면서, 전체 내용을 정리 및 활용하기 위한 마지막 프로젝트를 실습하면서 정리한 Repository이다.

Reference: [Final Project: Building a Multithreaded Web Server](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html)

이 프로젝트는 Rust 프로그래밍 언어를 사용해 기초적인 웹 서버를 구축 후 "hello"를 웹 브라우저에 출력하는 것이다. 웹 서버를 구축하기 위한 계획은 다음과 같다:

1. TCP와 HTTP를 간략히 학습
2. 소켓에서 TCP 연결을 수신
3. 적은 수의 HTTP request를 파싱
4. 적절한 HTTP response 생성
5. thread pool를 사용해 서버의 throughput을 개선

## Overview TCP and HTTP

**HTTP**: Hypertext Transfer Protocol  
**TCP**: Transmission Control Protocol

HTTP와 TCP 모두 request-response 프로토콜이다. 

request-response 프로토콜: 클라이언트가 request를 시작하면 서버는 request를 수신한 후 client에게 response를 제공

TCP는 한 서버에서 다른 서버로 전달되는 방법에 대한 디테일을 설명하지만 그 정보가 무엇인지는 특정하지 않는 lower-level 프로토콜이다.  
HTTP는 request와 response의 내용을 정의함으로써 TCP위에 구축된다. HTTP를 다른 프로토콜과 함께 사용하는 게 가능하긴 하지만 일반적으로 TCP를 통해 데이터를 전송한다.

이 프로젝트에서 주소로 `127.0.0.1:7878`를 사용한다. 콜론 앞은 IP 주소를 나타내며, 특히 여기서는 local computer를 나타낸다. 콜론 뒤의 `7878`은 port이다. port에 연결하는 동작을 보통 port에 바인딩한다고 표현한다. 두 프로그램이 동일 port를 수신하는 경우 바인딩은 실패한다.

연결: full request와 response 처리 - 클라이언트의 서버 연결, 서버의 response 생성, 서버의 연결 종료

### HTTP Request

HTTP는 텍스트 기반 프로토콜로 request는 아래와 같은 format이다:

```
Method Request-URI HTTP-Version CRLF
headers CRLF
message-body
```

첫 줄은 *request line*으로 클라이언트가 요청하고 있는 것에 대한 정보를 담고 있다. `Method`는 `GET`이나 `POST`와 같은 현재 사용된 *method*로, 클라리언트가 어떻게 요청했는지를 나타낸다. `GET` request는 정보 요청을 의미한다. `Request-URI`는 클라이언트가 요청하고 있는 *Uniform Resource Identifier (URI)*를 나타낸다. `HTTP-Version`은 클라이언트가 사용하고 있는 HTTP version이다. `CRLF`는 request line의 종료를 나타내며 *carriage return*과 *line feed*로 구성된다. 즉, `\r\n`이다.

request line 이후의 나머지는 `Host:`로 시작한다. 참고로 `GET` request는 body가 없다.

### HTTP Response

response는 아래와 같은 format이다:

```
HTTP-Version Status-Code Reason-Phrase CRLF
headers CRLF
message-body
```

첫 줄은 *status line*으로 HTTP version과 request 결과를 요악한 numeric status code, status code에 대한 텍스트 설명을 제공하는 reason phrase로 구성된다.

아래는 HTTP version 1.1, status code 200, OK reason phrase, no headers, no body로 구성된 response 예시이다:

```
HTTP/1.1 200 OK\r\n\r\n
```

## Thread Pool

*thread pool*은 task를 기다리고 다루도록 준비하기 위해 생성된 thread 그룹을 의미한다. 프로그램이 새로운 task를 받으면, thread pool 내의 한 thread에 task를 할당한 후 처리한다. 할당된 thread가 task를 처리하는 동안, thread pool에 남아있는 thread들은 여전히 이용 가능하다. 할당된 thread의 처리가 완료되면 idle thread pool에 반환되고, 새 task를 처리할 준비를 한다. thread pool은 연결을 동시에 처리할 수 있어 서버의 throughput을 증가시킬 수 있다.

연결을 처리하기 위해 request는 pool에 보내진다. pool은 들어오는 request에 대한 queue를 유지한다. pool 내의 각 thread는 queue로부터 request를 꺼낸 후 request를 처리하고 다시 다른 request를 꺼낸다. 이 구조에서 thread 개수인 `N`만큼의 request를 동시에 처리할 수 있다.