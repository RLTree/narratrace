#[cfg(test)]
fn drain_test_http_request(stream: &mut std::net::TcpStream) {
    use std::io::{Read, Write};
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut request = Vec::new();
    let (body_start, content_length, chunked) = loop {
        let mut buffer = [0_u8; 4096];
        let count = stream.read(&mut buffer).unwrap();
        request.extend_from_slice(&buffer[..count]);
        let Some(header_end) = request.windows(4).position(|value| value == b"\r\n\r\n") else {
            continue;
        };
        let headers = String::from_utf8_lossy(&request[..header_end]).to_ascii_lowercase();
        if headers.contains("expect: 100-continue") {
            stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n").unwrap();
        }
        let length = headers.lines().find_map(|line| {
            line.strip_prefix("content-length:")
                .and_then(|value| value.trim().parse::<usize>().ok())
        });
        break (header_end + 4, length, headers.contains("transfer-encoding: chunked"));
    };
    while !content_length.is_some_and(|length| request.len() >= body_start + length)
        && !(chunked && request.ends_with(b"\r\n0\r\n\r\n"))
    {
        let mut buffer = [0_u8; 4096];
        let count = stream.read(&mut buffer).unwrap();
        if count == 0 { break; }
        request.extend_from_slice(&buffer[..count]);
    }
}
