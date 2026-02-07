use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

fn main() {
    println!("HTTP CONNECT 代理测试工具\n");

    // 连接到代理
    let proxy_addr = "127.0.0.1:12345"; // 修改为实际的代理端口
    println!("连接到代理: {}", proxy_addr);

    let mut stream = match TcpStream::connect(proxy_addr) {
        Ok(s) => {
            println!("✓ 已连接到代理");
            s
        }
        Err(e) => {
            println!("✗ 连接失败: {}", e);
            println!("  提示: 确保应用已启动并且代理正在运行");
            return;
        }
    };

    // 设置超时
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("设置读取超时失败");
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .expect("设置写入超时失败");

    // 发送 CONNECT 请求
    let connect_request = "CONNECT www.baidu.com:443 HTTP/1.1\r\nHost: www.baidu.com:443\r\n\r\n";
    println!("\n发送 CONNECT 请求:");
    println!("  {}", connect_request.replace("\r\n", "\\r\\n"));

    if let Err(e) = stream.write_all(connect_request.as_bytes()) {
        println!("✗ 发送请求失败: {}", e);
        return;
    }

    println!("✓ 请求已发送");

    // 读取响应
    println!("\n等待响应...");
    let mut reader = BufReader::new(&stream);
    let mut response_line = String::new();

    if let Err(e) = reader.read_line(&mut response_line) {
        println!("✗ 读取响应失败: {}", e);
        return;
    }

    println!("✓ 收到响应:");
    println!("  {}", response_line.trim());

    // 分析响应
    if response_line.starts_with("HTTP/1.1 200") {
        println!("\n✓ 代理工作正常！连接已建立。");

        // 读取剩余的头部
        loop {
            let mut line = String::new();
            if let Err(e) = reader.read_line(&mut line) {
                println!("  读取头部时出错: {}", e);
                break;
            }
            if line.trim().is_empty() {
                break;
            }
            println!("  {}", line.trim());
        }

        println!("\n连接状态: 已建立隧道");
        println!("代理可以正常工作！");
    } else {
        println!("\n✗ 代理返回错误！");
        println!("  可能的原因:");
        println!("  1. SSH 连接失败");
        println!("  2. 目标主机无法访问");
        println!("  3. SSH 服务器未启用 AllowTcpForwarding");
    }
}
