// Avoid musl's default allocator due to lackluster performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{
    collections::{BTreeMap, HashMap},
    env,
    net::SocketAddr,
    str::FromStr,
};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use unic_emoji_char::is_emoji;
use warp::{
    http::StatusCode,
    reply::{Reply, Response},
    Filter,
};

#[derive(Deserialize)]
struct PostData {
    slug: String,
    target: String,
}

#[derive(Serialize)]
struct Reaction {
    target: String,
    reactions: i32,
    reacted: bool,
}

fn get_real_ip(headers: warp::http::HeaderMap, addr: Option<std::net::SocketAddr>) -> String {
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            return forwarded_str
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
        }
    }
    // Fallback to remote address
    addr.map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn hash_ip(ip: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn get_reactions(
    p: HashMap<String, String>,
    emojis: &str,
    uid: &str,
    pool: &Pool<SqliteConnectionManager>,
) -> Result<Response, Box<dyn std::error::Error>> {
    let slug = match p.get("slug") {
        Some(slug) => slug,
        None => {
            return Ok(
                warp::reply::with_status("Missing param: slug", StatusCode::BAD_REQUEST)
                    .into_response(),
            );
        }
    };

    let db = pool.get()?;
    let mut stmt = db.prepare(include_str!("./sql/get_reactions.sql"))?;

    let reactions = stmt.query_map(params![slug, uid], |row| {
        Ok(Reaction {
            target: row.get(0)?,
            reactions: row.get(1)?,
            reacted: row.get(2)?,
        })
    })?;

    let reactions: Vec<Reaction> = reactions.flatten().collect::<Vec<_>>();

    let mut response: BTreeMap<String, (i32, bool)> = BTreeMap::new();
    for emoji in emojis.chars() {
        if is_emoji(emoji) {
            response.insert(emoji.to_string(), (0, false));
        }
    }

    for reaction in &reactions {
        if let Some(target) = response.get_mut(&reaction.target) {
            target.0 = reaction.reactions;
            target.1 = reaction.reacted;
        }
    }

    Ok(warp::reply::json(&response).into_response())
}

fn post_reaction(
    reaction: &PostData,
    uid: &str,
    pool: &Pool<SqliteConnectionManager>,
) -> Result<Response, Box<dyn std::error::Error>> {
    if reaction.slug.trim() == "" {
        return Ok(warp::reply::with_status("Slug blank", StatusCode::BAD_REQUEST).into_response());
    }

    let char = match reaction.target.chars().next() {
        Some(char) => char,
        None => {
            return Ok(
                warp::reply::with_status("Target blank", StatusCode::BAD_REQUEST).into_response(),
            );
        }
    };

    if !is_emoji(char) {
        return Ok(
            warp::reply::with_status("Target is not an emoji", StatusCode::BAD_REQUEST)
                .into_response(),
        );
    }

    let db = pool.get()?;

    let mut stmt = db.prepare(include_str!("./sql/get_reaction_for_target.sql"))?;

    let result: Option<String> = stmt
        .query_row(params![reaction.slug, reaction.target, uid], |row| {
            row.get(0)
        })
        .optional()?;

    let reacted = result.is_some();

    if !reacted {
        db.execute(
            include_str!("./sql/create_reaction.sql"),
            params![reaction.slug, reaction.target, uid],
        )?;
    } else {
        db.execute(
            include_str!("./sql/delete_reaction.sql"),
            params![reaction.slug, reaction.target, uid],
        )?;
    }

    Ok(warp::reply::json(&("success", true)).into_response())
}

#[tokio::main]
async fn main() {
    let db_path = match env::var("REACTION_DB") {
        Ok(path) => path,
        Err(_) => "./reactions.db".to_string(),
    };

    let host = match env::var("REACTION_HOST") {
        Ok(host) => host,
        Err(_) => "0.0.0.0".to_string(),
    };

    let emojis = match env::var("REACTION_EMOJIS") {
        Ok(emojis) => emojis,
        Err(_) => "👍❤️🔥👏🤩😂😢🤔😡🤯🧠🎉⚡💤🤝".to_string(),
    };

    let port = match env::var("REACTION_PORT") {
        Ok(port) => match port.parse::<i32>() {
            Ok(port) => port,
            Err(e) => {
                panic!("Invalid port: {}", e);
            }
        },
        Err(_) => 8080,
    };

    let manager = SqliteConnectionManager::file(db_path);
    let pool = match r2d2::Pool::new(manager) {
        Ok(pool) => pool,
        Err(e) => {
            panic!("Error creating db pool: {}", e);
        }
    };

    {
        let db = match pool.get() {
            Ok(db) => db,
            Err(e) => {
                panic!("Failed to get db: {}", e);
            }
        };

        if let Err(e) = db.execute(include_str!("./sql/setup.sql"), ()) {
            panic!("Failed to initialize database: {}", e);
        }
    }

    let get_pool = pool.clone();
    let get = warp::get()
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::header::headers_cloned())
        .and(warp::addr::remote())
        .and(warp::any().map(move || get_pool.clone()))
        .map(
            move |p: HashMap<String, String>,
                  headers: warp::http::HeaderMap,
                  remote_addr: Option<std::net::SocketAddr>,
                  pool: Pool<SqliteConnectionManager>| match get_reactions(
                p,
                &emojis,
                &hash_ip(&get_real_ip(headers, remote_addr)),
                &pool,
            ) {
                Ok(reply) => reply,
                Err(e) => warp::reply::with_status(
                    format!("Error getting responses: {}", e),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response(),
            },
        );

    let post = warp::post()
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .and(warp::header::headers_cloned())
        .and(warp::addr::remote())
        .and(warp::any().map(move || pool.clone()))
        .map(
            |reaction: PostData,
             headers: warp::http::HeaderMap,
             remote_addr: Option<std::net::SocketAddr>,
             pool: Pool<SqliteConnectionManager>| match post_reaction(
                &reaction,
                &hash_ip(&get_real_ip(headers, remote_addr)),
                &pool,
            ) {
                Ok(response) => response,
                Err(e) => warp::reply::with_status(
                    format!("Error posting reaction: {}", e),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response(),
            },
        );

    let routes = get.or(post);

    let address = format!("{}:{}", host, port);

    println!("Reaction server running at: http://{}", address);

    let address = match SocketAddr::from_str(&address) {
        Ok(address) => address,
        Err(e) => {
            panic!("Failed to parse address ({}): {}", address, e);
        }
    };

    warp::serve(routes).run(address).await;
}
