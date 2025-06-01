# Godot 网页服务器 🚀

**专为 Godot Web 导出优化的 HTTPS 服务器**，支持智能音频模式切换、性能监控和开发者友好功能
![License](https://img.shields.io/badge/license-MIT-blue)

## ✨ 核心功能

- 🔒 **自动 HTTPS** - 自签名证书一键生成
- 🎵 **智能音频模式** - 自动切换 Worklet/ScriptProcessor
- 📊 **性能监控** - 控制台实时显示加载指标
- 🛠️ **开发者工具** - 调试模式自动检测
- 🌐 **跨域隔离** - 完美支持 Godot WebAssembly
- ⚡ **开箱即用** - 适配 Godot 3.5+/4.x 网页导出

## 🚀 快速开始

### 环境准备

- Rust 1.70+ (`rustup install stable`)
- Godot 3.5+ 或 4.x 网页导出

```bash
# 克隆并运行
git clonehttps://github.com/cfping/godot_https_server.git
cd godot-web-server
cargo run --release
```

访问你的游戏：
👉 [https://localhost:8443](https://localhost:8443)


### 服务器参数

| 参数                | 说明                     | 默认值  |
|---------------------|--------------------------|---------|
| `?audio=worklet`   | 强制使用 AudioWorklet    | 自动    |
| `?audio=legacy`    | 强制使用 ScriptProcessor | 自动    |
| `#debug`           | 启用性能日志             | 关闭    |

## 📦 部署流程

### 1. 导出游戏

```bash
godot --export-release "Web" 导出目录路径
```

### 2. 准备文件

```bash
cp -r 导出目录路径/* godot-web-server/
```

### 3. 生产环境运行

```bash
# 编译发布版
cargo build --release
# 启动服务（绑定443端口）
sudo ./target/release/godot-web-server
```

## 🎛️ 高级用法

### 自定义SSL证书

替换自动生成的证书：

```bash
# Let's Encrypt 示例
cp /etc/letsencrypt/live/yourdomain.com/{cert.pem,key.pem} .
```

### Docker 部署

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release
FROM debian:bullseye-slim
COPY --from=builder /app/target/release/godot-web-server /usr/local/bin/
COPY ./export /var/www
CMD ["godot-web-server"]
```

## 🧑‍💻 开发模式

### 启用调试

在HTML中添加标记：

```html
<!--DEVMODE-->
```

将激活：

- 实时重载检测
- 详细控制台日志
- 缓存绕过

### 音频模式测试

| 网址                                | 用途                  |
|-------------------------------------|-----------------------|
| `https://localhost:8443`           | 自动检测              |
| `https://localhost:8443?audio=worklet#debug` | 调试Worklet模式       |
| `https://localhost:8443?audio=legacy`       | 强制传统音频模式      |

## ⚠️ 常见问题

### 问题排查

1. **混合内容错误**
   确保所有资源使用 `https://`
2. **iOS音频问题**
   在Godot项目中添加：

   ```gdscript
   # 在自动加载脚本中
   func _ready():
       OS.set_environment("WEB_AUDIO_CONTEXT", "worklet" if OS.has_feature("web") else "")
   ```

3. **证书警告**
   开发环境手动信任证书：

   ```bash
   # Linux系统
   sudo cp cert.pem /usr/local/share/ca-certificates/godot-dev.crt
   sudo update-ca-certificates
   ```

## 📜 开源协议

MIT 许可证 - 详见 [LICENSE](LICENSE)

---
