# HTTP 代理测试指南

## 1. 启动应用

```bash
cd /Users/gaozhan/mycode/ceshi/SynapSH
pnpm tauri dev
```

## 2. 连接到 SSH 服务器

确保 SSH 服务器启用了 TCP 端口转发：
```bash
# 在服务器上检查
grep AllowTcpForwarding /etc/ssh/sshd_config
# 应该显示 AllowTcpForwarding yes
```

## 3. 打开浏览器

点击桌面上的浏览器图标，应该会：
1. 在本地启动 HTTP 代理（监听随机端口，如 127.0.0.1:12345）
2. 使用 `--proxy-server=http://127.0.0.1:12345` 启动 Chrome

## 4. 检查日志

在终端中查找以下日志：

### 成功的日志示例：
```
INFO 启动 Chrome，代理端口: 12345, URL: https://www.baidu.com
DEBUG HTTP CONNECT: www.baidu.com:443 from 127.0.0.1:xxxxx
DEBUG Proxy data transfer ended: <错误信息>
```

### 失败的日志示例：
```
ERROR Failed to open SSH channel to www.baidu.com:443: <错误信息>
```

## 5. 常见问题排查

### 问题 1：Chrome 无法连接
**可能原因**：
- HTTP 代理未正常启动
- 端口被占用

**解决方案**：
```bash
# 检查端口是否在监听
lsof -i :<代理端口>

# 查看 Chrome 日志（在 Chrome 中输入）
chrome://net-internals/#proxy
```

### 问题 2：连接超时
**可能原因**：
- SSH 服务器未启用 `AllowTcpForwarding`
- 防火墙阻止连接

**解决方案**：
```bash
# 在服务器上启用端口转发
sudo echo "AllowTcpForwarding yes" >> /etc/ssh/sshd_config
sudo systemctl restart sshd
```

### 问题 3：DNS 解析失败
**可能原因**：
- 远程服务器无法解析域名

**解决方案**：
- 在 `/etc/hosts` 中添加域名解析
- 或者配置远程服务器的 DNS 服务器

## 6. 手动测试代理

方法 1：使用 curl
```bash
# 需要知道代理端口（从日志中获取）
PROXY_PORT=<代理端口>
curl -x http://127.0.0.1:$PROXY_PORT https://www.baidu.com -v
```

方法 2：使用 nc (netcat)
```bash
PROXY_PORT=<代理端口>
echo -e "CONNECT www.baidu.com:443 HTTP/1.1\r\nHost: www.baidu.com:443\r\n\r\n" | nc 127.0.0.1 $PROXY_PORT
```

## 7. 性能对比

| 指标 | SOCKS5 | HTTP CONNECT |
|------|---------|--------------|
| 连接建立 | ~3-5 RTT | ~1-2 RTT |
| 协议复杂度 | 高（多步握手） | 低（单次请求） |
| 浏览器支持 | 完美 | 完美 |
| 调试难度 | 较难 | 容易 |

## 8. 进一步优化

如果 HTTP 代理仍然有问题，可以考虑：

1. **使用 HTTPS 代理**
   - 支持 HTTPS 站点的证书验证
   - 实现更复杂但更安全

2. **实现连接池**
   - 复用 SSH 通道
   - 减少连接建立开销

3. **使用内嵌浏览器**
   - Tauri WebView
   - 完全控制浏览器行为

## 9. 当前实现状态

✅ **已完成**：
- HTTP CONNECT 协议实现
- HTTP 头部解析
- 双向数据转发
- 错误处理和日志
- Chrome 启动参数优化

🔧 **待测试**：
- 实际连接测试
- 性能基准测试
- 稳定性测试
