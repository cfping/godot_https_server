use hyper::{server::conn::Http, service::service_fn, Body, Request, Response, StatusCode};
use std::{convert::Infallible, fs, net::SocketAddr, path::Path, sync::Arc};
use tokio::{fs::File, net::TcpListener};
use tokio_rustls::TlsAcceptor;
use rcgen::generate_simple_self_signed;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile;
use mime_guess::from_path;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 证书管理
    let cert_path = "cert.pem";
    let key_path = "key.pem";
    
    if !Path::new(cert_path).exists() || !Path::new(key_path).exists() {
        println!("⚠️ Generating self-signed certificate...");
        generate_self_signed_cert(cert_path, key_path)?;
        println!("✅ Certificate generated");
    }
    // 2. TLS配置
    let (certs, key) = load_tls_materials(cert_path, key_path)?;
    let tls_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    // 3. 服务启动
    let service = service_fn(handle_request);
    let tls_acceptor = Arc::new(TlsAcceptor::from(Arc::new(tls_config)));
    let tcp_listener = TcpListener::bind("0.0.0.0:8443").await?;
    
    println!("🚀 Server running at: https://localhost:8443");
    println!("🔉 Audio mode control: https://localhost:8443?audio=worklet|legacy");
    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;
        let tls_acceptor = tls_acceptor.clone();
        let service = service.clone();
        
        tokio::spawn(async move {
            match tls_acceptor.accept(tcp_stream).await {
                Ok(tls_stream) => {
                    if let Err(e) = Http::new()
                        .serve_connection(tls_stream, service)
                        .await {
                        eprintln!("Connection error: {}", e);
                    }
                }
                Err(e) => eprintln!("TLS handshake failed: {}", e),
            }
        });
    }
}
async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();
    let query = req.uri().query().unwrap_or_default();
    let file_path = match path {
        "/" => "index.html",
        _ => path.trim_start_matches('/'),
    };
    // 读取文件内容
    let file_content = tokio::fs::read_to_string(file_path).await;
    
    let mut response = match file_content {
        Ok(mut content) => {
            if file_path.ends_with(".html") {
                inject_audio_script(&mut content, query);
            }
            Response::new(Body::from(content))
        }
        Err(_) => match File::open(file_path).await {
            Ok(file) => {
                let stream = tokio_util::io::ReaderStream::new(file);
                Response::new(Body::wrap_stream(stream))
            }
            Err(_) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("404 Not Found"))
                .unwrap(),
        },
    };
    // 头信息设置
    let headers = response.headers_mut();
    headers.insert("Cross-Origin-Opener-Policy", "same-origin".parse().unwrap());
    headers.insert("Cross-Origin-Embedder-Policy", "require-corp".parse().unwrap());
    if let Some(mime) = from_path(file_path).first() {
        headers.insert("Content-Type", mime.to_string().parse().unwrap());
    }
    if !file_path.ends_with(".html") {
        headers.insert("Cache-Control", "public, max-age=86400".parse().unwrap());
    } else {
        headers.remove("Cache-Control"); // 开发阶段禁用HTML缓存
    }
    Ok(response)
}
fn inject_audio_script(html: &mut String, query_params: &str) {
    const AUDIO_SCRIPT: &str = r#"
    <script type="module">
    (() => {
        // 配置参数
        const config = {
            debug: true,
            defaultMode: 'worklet',
            fallbackTimeout: 1500,
            godot4Selector: '#godot-canvas',
            godot3Selector: 'body'
        };
        
        // 从URL参数解析设置
        const params = new URLSearchParams(location.search);
        const forceMode = params.get('audio');
        const useWorklet = forceMode 
            ? forceMode === 'worklet' 
            : config.defaultMode === 'worklet';
        
        // 性能监控
        const perfMark = (name) => config.debug && performance.mark(`audio_${name}`);
        
        // 初始化音频上下文
        const initAudio = () => {
            perfMark('init_start');
            
            const audio = document.createElement('audio');
            audio.setAttribute('context', useWorklet ? 'worklet' : 'scriptprocessor');
            
            // 自动检测Godot版本
            const mountPoint = document.querySelector(config.godot4Selector) 
                || document.querySelector(config.godot3Selector);
            
            if (mountPoint) {
                mountPoint.appendChild(audio);
                
                // Worklet错误处理
                if (useWorklet) {
                    audio.onerror = () => {
                        console.warn('[Audio] Worklet failed, falling back');
                        location.search = '?audio=legacy';
                    };
                }
                
                perfMark('init_end');
                if (config.debug) {
                    const measure = performance.measure(
                        'audio_init', 
                        'audio_init_start', 
                        'audio_init_end'
                    );
                    console.log(`[Audio] Initialized in ${measure.duration.toFixed(2)}ms`);
                }
            }
        };
        
        // 延迟初始化避免阻塞
        setTimeout(initAudio, config.fallbackTimeout);
        
        // Godot引擎加载事件
        if (typeof Engine !== 'undefined') {
            Engine.on('started', () => {
                console.log('[Audio] Godot engine ready');
                initAudio();
            });
        }
    })();
    </script>
    "#;
    if let Some(pos) = html.find("</head>") {
        html.insert_str(pos, AUDIO_SCRIPT);
        
        // 开发模式提示
        if html.contains("<!--DEVMODE-->") {
            html.insert_str(pos, r#"<script>console.log('[Dev] Audio script injected');</script>"#);
        }
    }
}
fn generate_self_signed_cert(cert_path: &str, key_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names)?;
    
    if let Some(parent) = Path::new(cert_path).parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(cert_path, cert.serialize_pem()?)?;
    fs::write(key_path, cert.serialize_private_key_pem())?;
    
    Ok(())
}
fn load_tls_materials(cert_path: &str, key_path: &str) -> Result<(Vec<Certificate>, PrivateKey), Box<dyn std::error::Error>> {
    let cert_file = fs::read(cert_path)?;
    let mut cert_reader = std::io::Cursor::new(cert_file);
    let certs = rustls_pemfile::certs(&mut cert_reader)?
        .into_iter()
        .map(Certificate)
        .collect();
    
    let key_bytes = fs::read(key_path)?;
    let mut key_reader = std::io::Cursor::new(key_bytes);
    
    let private_key = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
        .ok()
        .and_then(|mut keys| keys.pop())
        .or_else(|| {
            key_reader.set_position(0);
            rustls_pemfile::rsa_private_keys(&mut key_reader)
                .ok()
                .and_then(|mut keys| keys.pop())
        })
        .or_else(|| {
            key_reader.set_position(0);
            rustls_pemfile::ec_private_keys(&mut key_reader)
                .ok()
                .and_then(|mut keys| keys.pop())
        })
        .ok_or("Failed to parse private key")?;
    
    Ok((certs, PrivateKey(private_key)))
}
