# serve-rust

A high-performance, concurrent, and robust static site serving command-line utility written in Rust. This tool is designed to serve static websites, single-page applications (SPAs), and folder contents with optimal speed and security, serving as a drop-in replacement for the popular `serve` NPM package.

## Features

- **Fast & Concurrent:** Built on `axum` and `tokio`.
- **Flexible Configuration:** Supports `serve.json`, `serve.yaml`, and `serve.toml`.
- **Middleware Pipeline:** Headers, Redirects, Clean URLs, Trailing Slashes, and Rewrites.
- **Directory Listing:** Beautiful, responsive folder views.
- **Media Streaming:** Support for HTTP Range requests and high-performance disk streaming.
- **SPA Support:** Easy Single Page Application fallback with `-s` or `--single`.
- **Secure:** Optional symlink protection and efficient Weak ETag generation.
- **Developer Friendly:** Beautiful startup banner and automatic clipboard integration.

## Installation

```bash
cargo install --path .
```

## Usage

Serve the current directory:
```bash
serve-rust
```

Serve a specific directory on a custom port:
```bash
serve-rust ./public -p 5000
```

Enable SPA fallback:
```bash
serve-rust -s
```

Listen on a specific address:
```bash
serve-rust -l localhost:8080
```

## Configuration

You can configure `serve-rust` using a `serve.json`, `serve.yaml`, or `serve.toml` file.

Example `serve.json`:

```json
{
  "public": "public",
  "cleanUrls": true,
  "rewrites": [
    { "source": "/app/**", "destination": "/index.html" }
  ]
}
```

### Options

| Option | Description |
| --- | --- |
| `public` | Subdirectory to serve assets from. |
| `cleanUrls` | Remove `.html` extensions from URLs. |
| `trailingSlash` | Enforce or remove trailing slashes. |
| `rewrites` | Internal path redirections. |
| `redirects` | External/Internal HTTP redirections. |
| `headers` | Custom HTTP headers per path glob. |
| `directoryListing` | Enable/disable visual folder indexes (default: `true`). |
| `symlinks` | Allow resolving symlinks (default: `false`). |
| `etag` | Enable/disable ETag generation (default: `true`). |
| `ignore` | Array of globs to exclude from serving and listing. |
| `media` | Support for Range requests and streaming extensions. |

## License

MIT
