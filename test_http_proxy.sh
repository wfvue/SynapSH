#!/bin/bash
# 测试 HTTP 代理

echo "测试 HTTP CONNECT 代理..."
echo "发送 CONNECT 请求到本地代理..."

# 使用 nc (netcat) 发送 HTTP CONNECT 请求
echo -e "CONNECT www.baidu.com:443 HTTP/1.1\r\nHost: www.baidu.com:443\r\n\r\n" | nc 127.0.0.1 8888

echo ""
echo "如果看到 'HTTP/1.1 200 Connection Established'，则代理工作正常"
echo "如果看到错误或无响应，则需要检查代理实现"
