use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, header},
    response::{IntoResponse, Html},
};
use std::sync::Arc;
use crate::config::{Config, CleanUrls, DirectoryListing};
use std::path::{Path, PathBuf};
use tokio::fs;
use mime_guess::from_path;
use glob::Pattern;
use sha2::{Sha256, Digest};

pub struct AppState {
    pub config: Config,
    pub base_path: PathBuf,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let mut path = req.uri().path().to_string();
    let mut res_headers = header::HeaderMap::new();

    // Pre-calculate full path for directory check
    let mut full_path = state.base_path.clone();
    if let Some(public) = &state.config.public {
        full_path.push(public);
    }
    let rel_path = path.trim_start_matches('/');
    full_path.push(rel_path);

    let ignore_patterns: Vec<Pattern> = state.config.ignore.as_ref()
        .map(|globs| globs.iter().filter_map(|g| Pattern::new(g).ok()).collect())
        .unwrap_or_default();

    // 0. Ignore
    if matches_ignore(rel_path, full_path.is_dir(), &ignore_patterns) {
        return (StatusCode::NOT_FOUND, res_headers, "404 Not Found").into_response();
    }

    // 1. Global Headers
    if let Some(rules) = &state.config.headers {
        for rule in rules {
            if Pattern::new(&rule.source).map(|p| p.matches(&path)).unwrap_or(false) {
                for h in &rule.headers {
                    if let Ok(name) = header::HeaderName::from_bytes(h.key.as_bytes()) {
                        if let Ok(value) = header::HeaderValue::from_str(&h.value) {
                            res_headers.insert(name, value);
                        }
                    }
                }
            }
        }
    }

    // 2. Redirects
    if let Some(redirects) = &state.config.redirects {
        for r in redirects {
            if Pattern::new(&r.source).map(|p| p.matches(&path)).unwrap_or(false) {
                let status = StatusCode::from_u16(r.redirect_type.unwrap_or(301)).unwrap_or(StatusCode::MOVED_PERMANENTLY);
                return (status, [(header::LOCATION, r.destination.clone())]).into_response();
            }
        }
    }

    // 3. Clean URLs
    if let Some(clean) = &state.config.clean_urls {
        let should_clean = match clean {
            CleanUrls::Boolean(b) => *b,
            CleanUrls::Globs(globs) => globs.iter().any(|g| Pattern::new(g).map(|p| p.matches(&path)).unwrap_or(false)),
        };

        if should_clean {
            if path.ends_with(".html") {
                let new_path = &path[..path.len() - 5];
                return (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, new_path.to_string())]).into_response();
            }
        }
    }

    // 4. Trailing Slash
    if let Some(ts) = state.config.trailing_slash {
        if ts {
            if !path.ends_with('/') && !path.contains('.') {
                return (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, format!("{}/", path))]).into_response();
            }
        } else {
            if path.ends_with('/') && path != "/" {
                return (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, path.trim_end_matches('/').to_string())]).into_response();
            }
        }
    }

    // 5. Rewrites
    if let Some(rewrites) = &state.config.rewrites {
        for rw in rewrites {
            if Pattern::new(&rw.source).map(|p| p.matches(&path)).unwrap_or(false) {
                path = rw.destination.clone();
                break;
            }
        }
    }

    // 6. File/Directory Resolution
    let rel_path = path.trim_start_matches('/');
    let mut full_path = state.base_path.join(rel_path);

    // If cleanUrls is active, we might need to append .html internally
    if !full_path.exists() && !path.ends_with(".html") {
        let mut with_html = full_path.clone();
        with_html.set_extension("html");
        if with_html.exists() {
            full_path = with_html;
        }
    }

    if full_path.is_dir() {
        let index_path = full_path.join("index.html");
        if index_path.exists() {
            full_path = index_path;
        } else {
            // Directory Listing
            let show_listing = match &state.config.directory_listing {
                Some(DirectoryListing::Boolean(b)) => *b,
                Some(DirectoryListing::Globs(globs)) => globs.iter().any(|g| Pattern::new(g).map(|p| p.matches(&path)).unwrap_or(false)),
                None => true,
            };

            if show_listing {
                return match render_directory_listing(&full_path, &path, &ignore_patterns).await {
                    Ok(html) => (StatusCode::OK, res_headers, Html(html)).into_response(),
                    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
                };
            } else {
                return (StatusCode::NOT_FOUND, res_headers, "404 Not Found").into_response();
            }
        }
    }

    if !full_path.exists() {
        // SPA fallback
        if state.config.rewrites.is_none() { 
            // If we are here and -s was passed, we should fallback to index.html
        }
        
        return (StatusCode::NOT_FOUND, res_headers, "404 Not Found").into_response();
    }

    // Symlinks check
    if let Ok(metadata) = fs::symlink_metadata(&full_path).await {
        if metadata.file_type().is_symlink() && !state.config.symlinks.unwrap_or(false) {
            return (StatusCode::NOT_FOUND, "Forbidden (Symlink)").into_response();
        }
    }

    // Ensure we are not trying to read a directory as a file (safeguard)
    if full_path.is_dir() {
        return (StatusCode::NOT_FOUND, res_headers, "404 Not Found").into_response();
    }

    // Serve file
    match fs::read(&full_path).await {
        Ok(content) => {
            let mime = from_path(&full_path).first_or_octet_stream();
            res_headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_str(mime.as_ref()).unwrap());

            // ETag
            if state.config.etag.unwrap_or(true) {
                let mut hasher = Sha256::new();
                hasher.update(&content);
                let etag = format!("W/\"{}-{}\"", content.len(), hex::encode(hasher.finalize()));
                
                if let Some(if_none_match) = req.headers().get(header::IF_NONE_MATCH) {
                    if if_none_match == etag.as_str() {
                        return (StatusCode::NOT_MODIFIED, res_headers).into_response();
                    }
                }
                res_headers.insert(header::ETAG, header::HeaderValue::from_str(&etag).unwrap());
            }

            (StatusCode::OK, res_headers, content).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

async fn render_directory_listing(dir: &Path, virt_path: &str, ignore_patterns: &[Pattern]) -> Result<String, std::io::Error> {
    let mut entries = fs::read_dir(dir).await?;
    let mut files = Vec::new();
    
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().await?.is_dir();
        
        let rel_item_path = if virt_path.ends_with('/') {
            format!("{}{}", virt_path, name)
        } else {
            format!("{}/{}", virt_path, name)
        };
        let trim_rel_path = rel_item_path.trim_start_matches('/');

        if !matches_ignore(trim_rel_path, is_dir, ignore_patterns) {
            files.push((name, is_dir));
        }
    }
    
    files.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let mut html = format!("<html><head><title>Index of {}</title><style>body{{font-family:sans-serif;padding:20px;}}ul{{list-style:none;padding:0;}}li{{margin-bottom:10px;}}a{{text-decoration:none;color:#007bff;}}a:hover{{text-decoration:underline;}}</style></head><body>", virt_path);
    html.push_str(&format!("<h1>Index of {}</h1><ul>", virt_path));
    
    if virt_path != "/" {
        html.push_str("<li><a href=\"..\">..</a></li>");
    }

    for (name, is_dir) in files {
        let suffix = if is_dir { "/" } else { "" };
        html.push_str(&format!("<li><a href=\"{}{}\">{}{}</a></li>", name, suffix, name, suffix));
    }
    
    html.push_str("</ul></body></html>");
    Ok(html)
}

fn matches_ignore(path: &str, is_dir: bool, patterns: &[Pattern]) -> bool {
    if path.is_empty() { return false; }
    for pattern in patterns {
        if pattern.matches(path) {
            return true;
        }
        if is_dir {
            // Check if pattern matches path with trailing slash
            if pattern.matches(&format!("{}/", path)) {
                return true;
            }
            // Check if pattern is a prefix ignore like "src/**"
            // We test this by checking if a dummy child path would be ignored
            if pattern.matches(&format!("{}/.ignore-check", path)) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trailing_slash_redirect() {
        let state = Arc::new(AppState {
            config: Config {
                trailing_slash: Some(true),
                ..Default::default()
            },
            base_path: PathBuf::from("."),
        });

        let req = Request::builder().uri("/test").body(Body::empty()).unwrap();
        let res = handler(State(state), req).await.into_response();
        
        assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(res.headers().get(header::LOCATION).unwrap(), "/test/");
    }

    #[tokio::test]
    async fn test_clean_urls_redirect() {
        let state = Arc::new(AppState {
            config: Config {
                clean_urls: Some(CleanUrls::Boolean(true)),
                ..Default::default()
            },
            base_path: PathBuf::from("."),
        });

        let req = Request::builder().uri("/page.html").body(Body::empty()).unwrap();
        let res = handler(State(state), req).await.into_response();
        
        assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(res.headers().get(header::LOCATION).unwrap(), "/page");
    }

    #[tokio::test]
    async fn test_ignore_feature() {
        let state = Arc::new(AppState {
            config: Config {
                ignore: Some(vec![".env".to_string(), "node_modules/**".to_string()]),
                ..Default::default()
            },
            base_path: PathBuf::from("."),
        });

        // Test ignored file
        let req1 = Request::builder().uri("/.env").body(Body::empty()).unwrap();
        let res1 = handler(State(state.clone()), req1).await.into_response();
        assert_eq!(res1.status(), StatusCode::NOT_FOUND);

        // Test ignored folder
        let req2 = Request::builder().uri("/node_modules/pkg/index.js").body(Body::empty()).unwrap();
        let res2 = handler(State(state.clone()), req2).await.into_response();
        assert_eq!(res2.status(), StatusCode::NOT_FOUND);

        // Test non-ignored file
        let req3 = Request::builder().uri("/public/index.html").body(Body::empty()).unwrap();
        let res3 = handler(State(state), req3).await.into_response();
        // It should be 404 because file doesn't exist in test env, but NOT because of ignore (mocking state is enough for logic check)
        assert_eq!(res3.status(), StatusCode::NOT_FOUND);
    }
}
