# Godot Web Server 🚀

## ✨ Features

- 🔒 **Auto HTTPS** with self-signed certificate generation
- 🎵 **Smart Audio Mode** (Worklet/ScriptProcessor auto-fallback)
- 📊 **Performance Monitoring** with console metrics
- 🛠️ **Developer Tools** including debug mode detection
- 🌐 **Cross-Origin Isolation** headers for Godot WebAssembly
- ⚡ **Zero-config** for most Godot 3.x/4.x web exports

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (`rustup install stable`)
- Godot 3.5+ or 4.x web export

```bash
# Clone and run
git clone git clonehttps://github.com/cfping/godot_https_server.git
cd godot-web-server
cargo run --release
```

Your Godot game will be served at:
👉 [https://localhost:8443](https://localhost:8443)



### Server Options

| Parameter          | Description                          | Default |
|--------------------|--------------------------------------|---------|
| `?audio=worklet`  | Force AudioWorklet mode              | auto    |
| `?audio=legacy`   | Force ScriptProcessor mode           | auto    |
| `#debug`          | Enable performance logging           | off     |

## 📦 Deployment

### 1. Export Your Game

```bash
godot --export-release "Web" path/to/export/folder
```

### 2. Prepare Files

```bash
cp -r path/to/export/folder/* godot-web-server/
```

### 3. Production Build

```bash
# Build release binary
cargo build --release
# Run server (bind to port 443)
sudo ./target/release/godot-web-server
```

## 🎛️ Advanced Usage

### Custom SSL Certificates

Replace the auto-generated certs:

```bash
# Let's Encrypt example
cp /etc/letsencrypt/live/yourdomain.com/{cert.pem,key.pem} .
```

### Docker Deployment

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

## 🧑‍💻 Development

### Debug Mode

Add this comment to your HTML:

```html
<!--DEVMODE-->
```

Will enable:

- Live reload detection
- Extended console logging
- Cache bypass

### Testing Audio Modes

| URL | Purpose |
|-----|---------|
| `https://localhost:8443` | Auto-detection |
| `https://localhost:8443?audio=worklet#debug` | Debug Worklet mode |
| `https://localhost:8443?audio=legacy` | Force legacy audio |

## ⚠️ Troubleshooting

### Common Issues

1. **Mixed Content Errors**
   Ensure all resources use `https://`
2. **iOS Audio Issues**
   Add this to your Godot project:

   ```gdscript
   # In your autoload script
   func _ready():
       OS.set_environment("WEB_AUDIO_CONTEXT", "worklet" if OS.has_feature("web") else "")
   ```

3. **Certificate Warnings**
   For development, manually trust the cert:

   ```bash
   # Linux
   sudo cp cert.pem /usr/local/share/ca-certificates/godot-dev.crt
   sudo update-ca-certificates
   ```

## 📜 License

MIT License - See [LICENSE](LICENSE) for details

---