#![allow(deprecated)]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use assert_cmd::Command;
use predicates::prelude::*;

fn local_response(body: &'static str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "CLI did not reach the local host"
                    );
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept failed: {error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let count = stream.read(&mut buffer).unwrap();
            assert!(count > 0, "incomplete HTTP request");
            request.extend_from_slice(&buffer[..count]);
        }
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        String::from_utf8(request)
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .to_owned()
    });
    (format!("http://{address}"), handle)
}

#[test]
fn routes_all_services_to_prefixed_hosts_with_or_without_trailing_slashes() {
    let cases: &[(&str, &str, &[&str], &str, &str)] = &[
        (
            "POLYMARKET_GAMMA_HOST",
            "gamma",
            &["markets", "list", "--limit", "1"],
            "[]",
            "/gamma/markets",
        ),
        (
            "POLYMARKET_DATA_HOST",
            "data",
            &[
                "data",
                "positions",
                "0x1111111111111111111111111111111111111111",
            ],
            "[]",
            "/data/positions",
        ),
        (
            "POLYMARKET_CLOB_HOST",
            "clob",
            &["clob", "midpoint", "1001"],
            "{\"mid\":\"0.58\"}",
            "/clob/midpoint",
        ),
    ];
    for (variable, prefix, args, body, expected_path) in cases {
        for suffix in ["", "/", "///"] {
            let (base, request) = local_response(body);
            let mut command = Command::cargo_bin("polymarket").unwrap();
            // Any accidental use of an unrelated service fails without network access.
            for host in [
                "POLYMARKET_GAMMA_HOST",
                "POLYMARKET_DATA_HOST",
                "POLYMARKET_CLOB_HOST",
            ] {
                command.env(host, "invalid-url");
            }
            command
                .env(variable, format!("{base}/{prefix}{suffix}"))
                .env_remove("POLYMARKET_PRIVATE_KEY")
                .args(["-o", "json"])
                .args(*args)
                .timeout(Duration::from_secs(10))
                .assert()
                .success();
            let line = request.join().unwrap();
            let path = line
                .split_whitespace()
                .nth(1)
                .unwrap()
                .split('?')
                .next()
                .unwrap();
            assert_eq!(path, *expected_path);
        }
    }
}

#[test]
fn invalid_hosts_return_cli_errors_without_panicking() {
    for (variable, args) in [
        ("POLYMARKET_GAMMA_HOST", vec!["markets", "list"]),
        (
            "POLYMARKET_DATA_HOST",
            vec![
                "data",
                "positions",
                "0x1111111111111111111111111111111111111111",
            ],
        ),
        ("POLYMARKET_CLOB_HOST", vec!["clob", "midpoint", "1001"]),
    ] {
        Command::cargo_bin("polymarket")
            .unwrap()
            .env(variable, "invalid-url")
            .args(["-o", "json"])
            .args(args)
            .timeout(Duration::from_secs(10))
            .assert()
            .failure()
            .stdout(predicate::str::contains("error"))
            .stderr(predicate::str::contains("panicked").not());
    }
}
