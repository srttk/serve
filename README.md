# serve

A high-performance, concurrent, and robust static site serving command-line utility written in Rust. This tool is designed to serve static websites, single-page applications (SPAs), and folder contents with optimal speed and security, serving as a drop-in replacement for the popular `serve` NPM package.

## Features

- **Fast & Concurrent:** Built on `axum` and `tokio`.
- **Flexible Configuration:** Supports `serve.json`, `serve.yaml`, and `serve.toml`.
- **Middleware Pipeline:** Headers, Redirects, Clean URLs, Trailing Slashes, and Rewrites.
- **Directory Listing:** Beautiful, responsive folder views.
- **Media Streaming:** Support for HTTP Range requests and high-performance disk streaming.
- **SPA Support:** Easy Single Page Application fallback with `-s` or `--single`.
- **Authentication:** HTTP Basic Authentication support for specific paths or the entire site.
- **Secure:** Optional symlink protection and efficient Weak ETag generation.

## Installation

```bash
cargo install --path .
```

## Usage

Serve the current directory:
```bash
serve
```

Serve a specific directory on a custom port:
```bash
serve ./public -p 5000
```

Enable SPA fallback:
```bash
serve -s
```

Listen on a specific address:
```bash
serve -l localhost:8080
```

Initialize a default configuration:
```bash
serve --init [json|yaml|toml]
```

## Configuration

You can configure `serve` using a `serve.json`, `serve.yaml`, or `serve.toml` file.

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
| `stream` | Support for Range requests and streaming extensions. |
| `auth` | Basic Authentication rules for path protection. |

### `auth`

Protects specific paths or the entire site using HTTP Basic Authentication.

> [!NOTE]
> If multiple patterns match a request path, the most specific rule (the one with the longest `source` pattern) will be applied. This ensures that detailed rules like `/admin/**` take precedence over broader ones like `/**`, regardless of their order in the configuration file.

| Option | Type | Description |
| --- | --- | --- |
| `source` | `String` | Path glob pattern to protect (e.g., `/private/**`). |
| `username` | `String` | Authorized username. |
| `password` | `String` | Authorized password (optional if `env_password` is used). |
| `env_password` | `String` | Name of environment variable containing the password. |

Example:
```json
{
  "auth": [
    {
      "source": "/private/**",
      "username": "admin",
      "password": "secretpassword"
    }
  ]
}
```

### `stream`

Enables high-performance video and audio streaming by supporting HTTP Range requests and disk streaming.

| Option | Type | Description | Default |
| --- | --- | --- | --- |
| `streamExtensions` | `String[]` | File extensions to handle via streaming and Range requests. | `["mp4", "webm", "ogg", "mp3", "wav", "pdf", "mkv", "mov"]` |
| `enableRanges` | `Boolean` | Enable/disable HTTP Range request support. | `true` |

Example:
```json
{
  "stream": {
    "streamExtensions": ["mp4", "webm", "mov"],
    "enableRanges": true
  }
}
```

#### Benefits
- **Seeking:** Allows instant jumping to any part of a video or audio file.
- **Memory Efficiency:** Handles files of any size (GBs) with minimal RAM overhead.
- **Performance:** Faster "Time to First Byte" (TTFB) for media playback.

## License

MIT
