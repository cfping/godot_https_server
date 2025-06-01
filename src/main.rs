use hyper::{server::conn::Http, service::service_fn, Body, Request, Response, StatusCode};
use std::{convert::Infallible, fs, net::SocketAddr, path::Path, sync::Arc};
use tokio::{fs::File, net::TcpListener};
use tokio_rustls::TlsAcceptor;
use rcgen::generate_simple_self_signed;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile;
use mime_guess::from_path; // 新增：MIME类型自动检测
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 证书路径设置
    let cert_path = "cert.pem";
    let key_path = "key.pem";
    
    // 2. 自动生成证书（如果不存在）
    if !Path::new(cert_path).exists() || !Path::new(key_path).exists() {
        println!("⚠️ 未找到证书文件，正在生成新的自签名证书...");
        generate_self_signed_cert(cert_path, key_path)?;
        println!("✅ 已生成新的自签名证书");
    }
    
    // 3. 加载TLS材料
    let (certs, key) = load_tls_materials(cert_path, key_path)?;
    
    // 4. 配置TLS
    let tls_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    
    // 5. 创建服务
    let service = service_fn(handle_request);
    let tls_acceptor = Arc::new(TlsAcceptor::from(Arc::new(tls_config)));
    
    // 6. 启动服务器
    let tcp_listener = TcpListener::bind("0.0.0.0:8443").await?;
    println!("🚀 HTTPS服务器运行在: https://localhost:8443");
    
    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;
        let tls_acceptor = tls_acceptor.clone();
        let service = service.clone();
        
        tokio::spawn(async move {
            match tls_acceptor.accept(tcp_stream).await {
                Ok(tls_stream) => {
                    if let Err(e) = Http::new()
                        .serve_connection(tls_stream, service)
                        .await
                    {
                        eprintln!("🔴 连接处理错误: {}", e);
                    }
                }
                Err(e) => eprintln!("🔴 TLS握手失败: {}", e),
            }
        });
    }
}
async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();
    let file_path = match path {
        "/" => "index.html",
        _ => path.trim_start_matches('/'),
    };
    
    // 尝试打开文件
    let file_result = File::open(file_path).await;
    
    // 构建基础响应
    let mut response = match file_result {
        Ok(file) => {
            let stream = tokio_util::io::ReaderStream::new(file);
            Response::new(Body::wrap_stream(stream))
        }
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404 Not Found"))
            .unwrap(),
    };
    // 获取响应头的可变引用
    let headers = response.headers_mut();
    // 关键：添加Godot Web导出必需的安全头
    headers.insert(
        "Cross-Origin-Opener-Policy",
        "same-origin".parse().unwrap(),
    );
    headers.insert(
        "Cross-Origin-Embedder-Policy",
        "require-corp".parse().unwrap(),
    );
    // 优化：自动设置MIME类型
    if let Some(mime) = from_path(file_path).first() {
        headers.insert(
            "Content-Type",
            mime.to_string().parse().unwrap(),
        );
    }
    // 优化：静态资源缓存控制
    if !file_path.ends_with(".html") {
        headers.insert(
            "Cache-Control",
            "public, max-age=86400".parse().unwrap(), // 缓存1天
        );
    }
    Ok(response)
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
    // 加载证书
    let cert_file = fs::read(cert_path)?;
    let mut cert_reader = std::io::Cursor::new(cert_file);
    let certs = rustls_pemfile::certs(&mut cert_reader)?
        .into_iter()
        .map(Certificate)
        .collect();
    
    // 加载私钥
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
        .ok_or("无法解析私钥：不是有效的PKCS8/RSA/EC格式")?;
    
    Ok((certs, PrivateKey(private_key)))
}
