# 🚀 Reaction – A Simple Emoji Reaction API

[![Rust](https://github.com/sorokya/reaction/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/sorokya/reaction/actions/workflows/rust.yml)

A lightweight and fast emoji reaction API built with Rust and powered by [Warp](https://crates.io/crates/warp).

Designed to integrate seamlessly with the serene theme for [Zola](https://www.getzola.org/).

---


## 🛠️ How to Build

### Prerequisites

- Install [rust](https://rustup.rs)

### Build Instructions

```sh
cargo build --release
```

This will compile the project in release mode for better performance.

---

## 🚀 How to Run

Reaction API can be configured using the following environment variables:

| **Variable**    | **Description**               | **Default Value** |
|-----------------|-------------------------------|-------------------|
| `REACTION_DB`   | Path to the SQLite database   | `./reactions.db`  |
| `REACTION_HOST` | Host the API should listen on | `0.0.0.0`         |
| `REACTION_PORT` | Port the API should listen on | `8080`            |

The API will automatically create a SQLite database if one doesn't exist.

### Run the API

- Using Cargo (if built locally):
```sh
cargo run --release
```

- Using Prebuilt Binary:
Download the [latest release](https://github.com/sorokya/reaction/releases/latest) for your system and run:
```sh
./reaction
```

---

## 🐳 Running with Docker

A prebuilt Docker image is available at:
📦 `ghcr.io/sorokya/reaction`

### Run with Docker

```sh
docker run -p 8080:8080 -v $(pwd)/reactions.db:/reaction/reactions.db ghcr.io/sorokya/reaction:master
```

This will:
- Expose the API on port 8080
- Mount your reactions.db file for persistent storage

---

## 📜 API Endpoints

| **Method** | **Endpoint** | **Description**                  |
|------------|--------------|----------------------------------|
| `GET`      | /            | Fetch reactions for a given slug |
| `POST`     | /            | Add a reaction to a slug         |

---

## 📄 License

This project is licensed under the MIT License. See [LICENSE](https://github.com/sorokya/reaction/blob/master/LICENSE.txt) for details.
