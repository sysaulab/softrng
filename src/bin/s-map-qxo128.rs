use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use rusqlite::{Connection, params};
use sha2::{Sha256, Digest};
use softrng::qxo128::{Qxo128, MAP_SIZE, Q128Map};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
//use std::io::Read;
use std::io::Write;

#[derive(Parser)]
#[command(name = "s-map-qxo128", about = "Redundant, verified QXO128 map manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new map from OS entropy or a seed file
    New {
        name: String,
        #[arg(short, long)]
        seed_file: Option<PathBuf>,
    },
    /// Delete a map
    Rm { name: String },
    /// Output 16‑byte chunks from a map
    Get {
        name: String,
        offset: String,
        chunks: Option<u128>,
    },
    /// List all maps
    List,
    /// Verify map integrity (redundant copies)
    Check { name: String },
}

fn db_path() -> PathBuf {
    dirs::home_dir().unwrap().join(".qxo128").join("maps.db")
}

fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS maps (
            name TEXT PRIMARY KEY,
            created_at INTEGER NOT NULL,
            copy0 BLOB NOT NULL,
            hash0 BLOB NOT NULL,
            copy1 BLOB NOT NULL,
            hash1 BLOB NOT NULL,
            copy2 BLOB NOT NULL,
            hash2 BLOB NOT NULL
        )",
        [],
    )?;
    Ok(())
}

fn cmd_new(name: String, seed_file: Option<PathBuf>) -> Result<()> {
    let db_dir = db_path().parent().unwrap().to_path_buf();
    std::fs::create_dir_all(&db_dir)?;
    let conn = Connection::open(db_path())?;
    init_db(&conn)?;

    let exists: bool = conn.query_row(
        "SELECT 1 FROM maps WHERE name = ?1",
        params![name],
        |_| Ok(true),
    ).unwrap_or(false);
    if exists {
        bail!("Map '{}' already exists", name);
    }

    let map = if let Some(seed_path) = seed_file {
        Q128Map::from_file(&seed_path)?
    } else {
        Q128Map::generate_with_seedy()?
    };

    let data = map.as_bytes().to_vec();
    let hash0 = Sha256::digest(&data);
    let hash1 = Sha256::digest(&data);
    let hash2 = Sha256::digest(&data);
    let created_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    conn.execute(
        "INSERT INTO maps (name, created_at, copy0, hash0, copy1, hash1, copy2, hash2)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            name,
            created_at,
            data.as_slice(),
            hash0.as_slice(),
            data.as_slice(),
            hash1.as_slice(),
            data.as_slice(),
            hash2.as_slice(),
        ],
    )?;
    println!("Map '{}' created successfully.", name);
    Ok(())
}

fn load_map(conn: &Connection, name: &str) -> Result<Qxo128> {
    let (copy0, hash0, copy1, hash1, copy2, hash2) = conn.query_row(
        "SELECT copy0, hash0, copy1, hash1, copy2, hash2 FROM maps WHERE name = ?1",
        params![name],
        |row| {
            Ok((
                row.get::<_, Vec<u8>>(0)?,
                row.get::<_, Vec<u8>>(1)?,
                row.get::<_, Vec<u8>>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, Vec<u8>>(4)?,
                row.get::<_, Vec<u8>>(5)?,
            ))
        },
    )?;

    for (data, expected_hash) in [(copy0, hash0), (copy1, hash1), (copy2, hash2)] {
        if data.len() != MAP_SIZE {
            continue;
        }
        let computed_hash = Sha256::digest(&data);
        if computed_hash.as_slice() == expected_hash {
            return Ok(Qxo128::new(Q128Map::from_bytes(&data)?));
        }
    }
    bail!("All copies of map '{}' are corrupted.", name);
}

fn cmd_rm(name: String) -> Result<()> {
    let conn = Connection::open(db_path())?;
    let changes = conn.execute("DELETE FROM maps WHERE name = ?1", params![name])?;
    if changes == 0 {
        bail!("Map '{}' not found.", name);
    }
    println!("Map '{}' removed.", name);
    Ok(())
}

fn cmd_get(name: String, offset_str: String, chunks: Option<u128>) -> Result<()> {
    let conn = Connection::open(db_path())?;
    let qxo = load_map(&conn, &name)?;
    let mut stdout = std::io::stdout().lock();
    let mut idx = parse_offset(&offset_str)?;

    match chunks {
        Some(count) => {
            for _ in 0..count {
                let chunk = qxo.read_chunk(idx);
                stdout.write_all(&chunk)?;
                idx = idx.wrapping_add(1);
            }
        }
        None => loop {
            let chunk = qxo.read_chunk(idx);
            if stdout.write_all(&chunk).is_err() {
                break;
            }
            idx = idx.wrapping_add(1);
        },
    }
    Ok(())
}

fn cmd_list() -> Result<()> {
    let conn = Connection::open(db_path())?;
    let mut stmt = conn.prepare("SELECT name, created_at FROM maps ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    for row in rows {
        let (name, created) = row?;
        println!("{} (created {})", name, created);
    }
    Ok(())
}

fn cmd_check(name: String) -> Result<()> {
    let conn = Connection::open(db_path())?;
    let (copy0, hash0, copy1, hash1, copy2, hash2) = conn.query_row(
        "SELECT copy0, hash0, copy1, hash1, copy2, hash2 FROM maps WHERE name = ?1",
        params![name],
        |row| {
            Ok((
                row.get::<_, Vec<u8>>(0)?,
                row.get::<_, Vec<u8>>(1)?,
                row.get::<_, Vec<u8>>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, Vec<u8>>(4)?,
                row.get::<_, Vec<u8>>(5)?,
            ))
        },
    )?;

    let copies = [(copy0, hash0), (copy1, hash1), (copy2, hash2)];
   
    for (i, (data, hash)) in copies.iter().enumerate() {
        if data.len() != std::mem::size_of::<Q128Map>() {
            println!("Copy {}: INVALID (wrong size)", i);
        } else {
            let computed = Sha256::digest(data);
            if computed.as_slice() == hash {
                println!("Copy {}: OK", i);
            } else {
                println!("Copy {}: CORRUPT (hash mismatch)", i);
            }
        }
    }
    Ok(())
}

fn parse_offset(s: &str) -> Result<u128> {
    if let Some(hex) = s.strip_prefix("0x") {
        u128::from_str_radix(hex, 16).context("Invalid hex offset")
    } else {
        s.parse::<u128>().context("Invalid decimal offset")
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name, seed_file } => cmd_new(name, seed_file),
        Commands::Rm { name } => cmd_rm(name),
        Commands::Get { name, offset, chunks } => cmd_get(name, offset, chunks),
        Commands::List => cmd_list(),
        Commands::Check { name } => cmd_check(name),
    }
}