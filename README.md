# 🚀 FastScout

> An asynchronous web scraper and page archive, written in Rust.

## 📌 About The Project

**FastScout** is a command-line tool that fetches a web page, extracts its title, visible text, and links, then stores the result in SQLite. The HTTP client and database pool run on Tokio. The finished program is one binary; it does not need Python or Node.js installed to run.

---

## ⚡ Key Features

* **Asynchronous I/O:** `tokio` and `reqwest` fetch the page without blocking the runtime.
* **Single binary:** `cargo build --release` produces one executable.
* **DOM parsing:** `scraper` reads the title, skips `script` / `style` text, and collects absolute links.
* **SQLite archive:** `sqlx` creates `pages` and upserts each URL with its title and body.
* **Clear failures:** `anyhow` prints the error chain. HTTP errors and invalid URLs stop the process with a non-zero exit code.
* **CLI output:** `clap` accepts `--url`, `--format`, and `--db`. Text mode is colorized with `colored`. JSON mode prints plain JSON so it can be piped.

---

## 🛠️ Tech Stack

* **Language:** [Rust](https://www.rust-lang.org/) (Edition 2021)
* **Async runtime:** `tokio`
* **HTTP client:** `reqwest`
* **HTML parser:** `scraper`
* **Database:** `sqlx` (SQLite)
* **CLI parser:** `clap`
* **Errors:** `anyhow`
* **Serialization:** `serde` and `serde_json`
* **Terminal color:** `colored`

---

## ⚙️ Installation & Getting Started

Install Rust and Cargo from [rustup.rs](https://rustup.rs/). On Windows, the SQLite driver compiles bundled SQLite, so the Visual Studio C++ build tools are required for the first build.

1. **Clone the repository:**

   ```bash
   git clone https://github.com/SphereXLabs/fastscout.git
   cd fastscout
   ```

2. **Build the project in release mode:**

   ```bash
   cargo build --release
   ```

3. **Run the binary:**

   ```bash
   ./target/release/fastscout --url "https://example.com" --format json
   ```

   On Windows:

   ```powershell
   .\target\release\fastscout.exe --url "https://example.com" --format json
   ```

A successful run writes `fastscout.db` in the working directory. Pass `--db path\to\archive.db` to choose another file.

---

## 💡 Usage

```text
fastscout --url <URL> --format <text|json> --db <PATH>
```

| Flag | Required | Default | Purpose |
| --- | --- | --- | --- |
| `--url` | yes | — | Page to fetch |
| `--format` | no | `text` | `text` for a colorized report, `json` for pretty JSON |
| `--db` | no | `fastscout.db` | SQLite file |

Text mode prints the URL, title, link count, each link, and the visible page text. JSON mode prints an object with `url`, `title`, `text`, and `links`.

```bash
cargo run -- --url "https://example.com" --format text
cargo run -- --url "https://example.com" --format json
```

Example JSON shape:

```json
{
  "url": "https://example.com/",
  "title": "Example Domain",
  "text": "Example Domain\nThis domain is for use in documentation examples...",
  "links": [
    "https://iana.org/domains/example"
  ]
}
```

The `pages` table stores `url` (unique), `title`, `body`, and `fetched_at`. Fetching the same URL again updates that row.

---

## 🗺️ Roadmap

- [x] SQLite persistence with `sqlx`
- [ ] Fetch many URLs concurrently
- [ ] Rate limiting and user-agent rotation
- [ ] Export saved pages to CSV

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome. Open an issue on the [repository](https://github.com/SphereXLabs/fastscout).

---

## 📜 License

Distributed under the MIT License. See [LICENSE](LICENSE) for the full text.
