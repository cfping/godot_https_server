# Godot Web Server 🚀

## Live Demo

Access our test server at:
👉 [https://114.55.90.171:8443](https://114.55.90.171:8443)

## ✨ Key Features

- 🔒 **Auto HTTPS** - Self-signed certificate generation
- 🎵 **Smart Audio Mode** - Auto-switch between Worklet/ScriptProcessor
- 📊 **Performance Monitoring** - Real-time console metrics
- 🛠️ **Developer Tools** - Debug mode detection
- 🌐 **Cross-Origin Isolation** - Full WebAssembly support
- ⚡ **Zero Configuration** - Works with Godot 3.5+/4.x web exports

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (`rustup install stable`)
- Godot 3.5+ or 4.x with Web export

```bash
# Clone and run
git clone https://github.com/cfping/godot_https_server.git
cd godot-web-server
cargo run --release
```

Access your game at:
👉 [https://localhost:8443](https://localhost:8443)

### Server Parameters

| Parameter          | Description                  | Default |
|--------------------|------------------------------|---------|
| `?audio=worklet`  | Force AudioWorklet mode      | Auto    |
| `?audio=legacy`   | Force ScriptProcessor mode   | Auto    |
| `#debug`          | Enable performance logging   | Off     |

## 📦 Deployment

### 1. Export Your Game

```bash
godot --export-release "Web" /path/to/export
```

### 2. Prepare Files

```bash
cp -r /path/to/export/* godot-web-server/
```

### 3. Production Run

```bash
# Build release
cargo build --release
# Start server (port 443)
sudo ./target/release/godot-web-server
```

## 🎛️ Advanced Usage

### Custom SSL Certificates

Replace auto-generated certs:

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

## 🧑‍💻 Development Mode

### Enable Debugging

Add to your HTML:

```html
<!--DEVMODE-->
```

Enables:

- Live reload
- Detailed console logs
- Cache bypass

### Audio Mode Testing

| URL | Purpose |
|-----|---------|
| `https://localhost:8443` | Auto-detect |
| `https://localhost:8443?audio=worklet#debug` | Debug Worklet |
| `https://localhost:8443?audio=legacy` | Force legacy |

## ⚠️ Troubleshooting

### Common Issues

1. **Mixed Content Errors**
   Ensure all resources use `https://`
2. **iOS Audio Issues**
   Add to your Godot project:

   ```gdscript
   # In autoload script
   func _ready():
       OS.set_environment("WEB_AUDIO_CONTEXT", "worklet" if OS.has_feature("web") else "")
   ```

3. **Certificate Warnings**
   For development, trust the cert manually:

   ```bash
   # Linux systems
   sudo cp cert.pem /usr/local/share/ca-certificates/godot-dev.crt
   sudo update-ca-certificates
   ```

## 📜 License

MIT License - See [LICENSE](LICENSE)
---

### Testing Notes

Try these test cases on our demo server:

1. **Basic Functionality**
   - Verify HTTPS connection
   - Check cross-origin isolation
   - Monitor audio mode selection in console
2. **Performance Test**

   ```javascript
   console.time('Godot Load');
   window.addEventListener('godot-loaded', () => {
     console.timeEnd('Godot Load');
     console.log('Memory usage:', performance.memory);
   });
   ```

3. **Audio Mode Tests**
   - [Worklet Mode](https://114.55.90.171:8443?audio=worklet)
   - [Legacy Mode](https://114.55.90.171:8443?audio=legacy)
4. **Debug Mode**
   Append `#debug` to URL for detailed logs
Report issues with:

1. Browser version

2. Console errors

3. Network timing screenshots

4. Reproduction steps

Note: Demo server uses self-signed certs - accept the security warning when testing.
