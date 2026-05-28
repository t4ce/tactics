use serde_json::Value;
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

pub const DEFAULT_SERVER_ADDR: &str = "trueos.eu:1337";
pub const DEFAULT_GAME: &str = "tactics";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Lobby {
    pub id: u64,
    pub name: String,
    pub game: String,
    pub players: u32,
    pub max_players: u32,
    pub status: String,
}

pub(super) struct TacticsClient {
    stream: TcpStream,
}

fn connection_closed_error(context: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::ConnectionAborted,
        format!("{context}: server closed connection without a response"),
    )
}

fn timeout_error(context: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        format!("{context}: server did not respond before timeout"),
    )
}

fn log_lobby_client(message: impl std::fmt::Display) {
    eprintln!("[lobby-client] {message}");
}

fn lobby_session_id() -> &'static str {
    static SESSION_ID: OnceLock<String> = OnceLock::new();
    SESSION_ID.get_or_init(|| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("tactics-{}-{now:x}", std::process::id())
    })
}

fn send(stream: &mut TcpStream, value: serde_json::Value) -> std::io::Result<()> {
    log_lobby_client(format_args!("send {value}"));
    write_json_line(stream, &value)
}

fn write_json_line(writer: &mut impl Write, value: &Value) -> std::io::Result<()> {
    let mut bytes = serde_json::to_vec(value).map_err(std::io::Error::other)?;
    bytes.push(b'\n');
    writer.write_all(&bytes)?;
    writer.flush()
}

pub(super) fn get_lobbies() -> std::io::Result<Vec<Lobby>> {
    get_lobbies_from(DEFAULT_SERVER_ADDR, DEFAULT_GAME)
}

#[allow(dead_code)]
pub(super) fn create_game_and_get_lobbies() -> std::io::Result<Vec<Lobby>> {
    create_game_and_get_lobbies_from(DEFAULT_SERVER_ADDR, DEFAULT_GAME, "Tactics lobby", 4)
}

pub(super) fn create_game_and_get_lobbies_with_created() -> std::io::Result<(Vec<Lobby>, Lobby)> {
    create_game_and_get_lobbies_with_created_from(
        DEFAULT_SERVER_ADDR,
        DEFAULT_GAME,
        "Tactics lobby",
        4,
    )
}

pub(super) fn get_lobbies_from(
    addr: impl ToSocketAddrs,
    game: &str,
) -> std::io::Result<Vec<Lobby>> {
    log_lobby_client(format_args!("refresh lobbies game={game}"));
    let mut client = TacticsClient::connect(addr)?;
    client.get_lobbies()
}

#[allow(dead_code)]
pub(super) fn create_game_from(
    addr: impl ToSocketAddrs,
    game: &str,
    name: &str,
    max_players: u32,
) -> std::io::Result<Lobby> {
    log_lobby_client(format_args!(
        "create game game={game} name={name:?} max_players={max_players}"
    ));
    let mut client = TacticsClient::connect(addr)?;
    client.create_game(name, game, max_players)?;
    client.wait_for_create_game_ack()?.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::TimedOut, "server did not confirm created game")
    })
}

#[allow(dead_code)]
pub(super) fn create_game_and_get_lobbies_from(
    addr: impl ToSocketAddrs,
    game: &str,
    name: &str,
    max_players: u32,
) -> std::io::Result<Vec<Lobby>> {
    log_lobby_client(format_args!(
        "create game then refresh game={game} name={name:?} max_players={max_players}"
    ));
    let mut client = TacticsClient::connect(addr)?;
    client.create_game(name, game, max_players)?;
    match client.wait_for_create_game_ack()? {
        Some(lobby) => log_lobby_client(format_args!("create game ack parsed lobby={lobby:?}")),
        None => log_lobby_client("create game ack missing or timed out; refreshing lobbies"),
    }
    client.get_lobbies()
}

pub(super) fn create_game_and_get_lobbies_with_created_from(
    addr: impl ToSocketAddrs,
    game: &str,
    name: &str,
    max_players: u32,
) -> std::io::Result<(Vec<Lobby>, Lobby)> {
    log_lobby_client(format_args!(
        "create game with ack then refresh game={game} name={name:?} max_players={max_players}"
    ));
    let mut client = TacticsClient::connect(addr)?;
    client.create_game(name, game, max_players)?;
    let lobby = client.wait_for_create_game_ack()?.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::TimedOut, "server did not confirm created game")
    })?;
    let lobbies = client.get_lobbies()?;
    Ok((lobbies, lobby))
}

pub(super) fn free_game_and_get_lobbies(game_id: u64) -> std::io::Result<Vec<Lobby>> {
    free_game_and_get_lobbies_from(DEFAULT_SERVER_ADDR, DEFAULT_GAME, game_id)
}

pub(super) fn free_game_and_get_lobbies_from(
    addr: impl ToSocketAddrs + Clone,
    game: &str,
    game_id: u64,
) -> std::io::Result<Vec<Lobby>> {
    log_lobby_client(format_args!("free game game={game} game_id={game_id}"));
    let mut client = TacticsClient::connect(addr.clone())?;
    client.free_game(game_id)?;
    client.wait_for_action_ack("free_game", "free_game")?;
    client.get_lobbies()
}

impl TacticsClient {
    pub fn connect(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
        log_lobby_client("connecting to lobby server");
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        log_lobby_client("connected to lobby server");
        Ok(Self { stream })
    }

    #[allow(dead_code)]
    pub fn hello(&mut self, name: &str, game: &str) -> std::io::Result<()> {
        send(
            &mut self.stream,
            json!({
                "type": "hello",
                "name": name,
                "ping_ms": 0,
                "latency_ms": 0,
                "game": game,
                "session_id": lobby_session_id()
            }),
        )
    }

    pub fn get_lobbies(&mut self) -> std::io::Result<Vec<Lobby>> {
        send(&mut self.stream, json!({"type": "game_list"}))?;

        let mut read = BufReader::new(self.stream.try_clone()?);
        let started = Instant::now();
        let mut line = String::new();
        loop {
            if started.elapsed() > Duration::from_secs(3) {
                log_lobby_client("game_list wait exceeded 3s");
                return Err(timeout_error("game_list"));
            }

            line.clear();
            match read.read_line(&mut line) {
                Ok(0) => {
                    log_lobby_client("game_list server closed connection");
                    return Err(connection_closed_error("game_list"));
                }
                Ok(_) => {
                    let raw = line.trim();
                    log_lobby_client(format_args!("recv game_list {raw}"));
                    let Ok(value) = serde_json::from_str::<Value>(raw) else {
                        log_lobby_client(format_args!("ignored non-json game_list frame {raw:?}"));
                        continue;
                    };
                    if value.get("type").and_then(Value::as_str) == Some("error") {
                        let message = value
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("server returned an error");
                        log_lobby_client(format_args!("game_list server error {message:?}"));
                        return Err(std::io::Error::other(message.to_owned()));
                    }
                    if let Some(lobbies) = lobbies_from_value(&value) {
                        log_lobby_client(format_args!("parsed {} lobbies", lobbies.len()));
                        return Ok(lobbies);
                    }
                    log_lobby_client(format_args!(
                        "ignored game_list frame without lobbies {value}"
                    ));
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    log_lobby_client(format_args!("game_list read timed out ({:?})", error.kind()));
                    return Err(timeout_error("game_list"));
                }
                Err(error) => {
                    log_lobby_client(format_args!("game_list read failed: {error}"));
                    return Err(error);
                }
            }
        }
    }

    pub fn create_game(&mut self, name: &str, game: &str, max_players: u32) -> std::io::Result<()> {
        send(
            &mut self.stream,
            json!({
                "type": "create_game",
                "name": name,
                "game": game,
                "max_players": max_players,
                "session_id": lobby_session_id()
            }),
        )
    }

    pub fn free_game(&mut self, game_id: u64) -> std::io::Result<()> {
        send(
            &mut self.stream,
            json!({
                "type": "free_game",
                "game_id": game_id,
                "session_id": lobby_session_id()
            }),
        )
    }

    fn wait_for_action_ack(&mut self, action: &str, context: &str) -> std::io::Result<()> {
        let mut read = BufReader::new(self.stream.try_clone()?);
        let started = Instant::now();
        let mut line = String::new();

        loop {
            if started.elapsed() > Duration::from_secs(2) {
                log_lobby_client(format_args!("{context} ack wait exceeded 2s"));
                return Err(timeout_error(context));
            }

            line.clear();
            match read.read_line(&mut line) {
                Ok(0) => {
                    log_lobby_client(format_args!("{context} ack connection closed"));
                    return Err(connection_closed_error(context));
                }
                Ok(_) => {
                    let raw = line.trim();
                    log_lobby_client(format_args!("recv {context}_ack {raw}"));
                    let Ok(value) = serde_json::from_str::<Value>(raw) else {
                        log_lobby_client(format_args!(
                            "ignored non-json {context}_ack frame {raw:?}"
                        ));
                        continue;
                    };
                    if value.get("type").and_then(Value::as_str) == Some("error") {
                        let message = value
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("server returned an error");
                        log_lobby_client(format_args!("{context} server error {message:?}"));
                        return Err(std::io::Error::other(message.to_owned()));
                    }
                    if value.get("type").and_then(Value::as_str) == Some("ack")
                        && value.get("action").and_then(Value::as_str) == Some(action)
                    {
                        return Ok(());
                    }
                    log_lobby_client(format_args!("ignored {context}_ack frame {value}"));
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    log_lobby_client(format_args!(
                        "{context} ack read timed out ({:?})",
                        error.kind()
                    ));
                    return Err(timeout_error(context));
                }
                Err(error) => {
                    log_lobby_client(format_args!("{context} ack read failed: {error}"));
                    return Err(error);
                }
            }
        }
    }

    fn wait_for_create_game_ack(&mut self) -> std::io::Result<Option<Lobby>> {
        let mut read = BufReader::new(self.stream.try_clone()?);
        let started = Instant::now();
        let mut line = String::new();

        loop {
            if started.elapsed() > Duration::from_secs(2) {
                log_lobby_client("create game ack wait exceeded 2s");
                return Err(timeout_error("create_game"));
            }

            line.clear();
            match read.read_line(&mut line) {
                Ok(0) => {
                    log_lobby_client("create game ack connection closed");
                    return Err(connection_closed_error("create_game"));
                }
                Ok(_) => {
                    let raw = line.trim();
                    log_lobby_client(format_args!("recv create_game_ack {raw}"));
                    let Ok(value) = serde_json::from_str::<Value>(raw) else {
                        log_lobby_client(format_args!(
                            "ignored non-json create_game_ack frame {raw:?}"
                        ));
                        continue;
                    };
                    if value.get("type").and_then(Value::as_str) == Some("error") {
                        let message = value
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("server returned an error");
                        log_lobby_client(format_args!("create_game server error {message:?}"));
                        return Err(std::io::Error::other(message.to_owned()));
                    }
                    if let Some(mut lobbies) = lobbies_from_value(&value) {
                        if let Some(lobby) = lobbies.pop() {
                            log_lobby_client(format_args!("parsed create_game ack {lobby:?}"));
                            return Ok(Some(lobby));
                        }
                    }
                    log_lobby_client(format_args!(
                        "ignored create_game_ack frame without lobby {value}"
                    ));
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    log_lobby_client(format_args!(
                        "create_game ack read timed out ({:?})",
                        error.kind()
                    ));
                    return Err(timeout_error("create_game"));
                }
                Err(error) => {
                    log_lobby_client(format_args!("create_game ack read failed: {error}"));
                    return Err(error);
                }
            }
        }
    }
}

fn lobbies_from_value(value: &Value) -> Option<Vec<Lobby>> {
    if let Some(items) = value.as_array() {
        return Some(items.iter().filter_map(Lobby::from_value).collect());
    }

    if value.get("type").and_then(Value::as_str) == Some("game_created") {
        return value
            .get("game")
            .and_then(Lobby::from_value)
            .map(|lobby| vec![lobby]);
    }

    for key in ["games", "lobbies", "game_list"] {
        if let Some(items) = value.get(key).and_then(Value::as_array) {
            return Some(items.iter().filter_map(Lobby::from_value).collect());
        }
    }

    if let Some(data) = value.get("data") {
        if let Some(lobbies) = lobbies_from_value(data) {
            return Some(lobbies);
        }
    }

    Lobby::from_value(value).map(|lobby| vec![lobby])
}

impl Lobby {
    fn from_value(value: &Value) -> Option<Self> {
        if let Some(kind) = value.get("type").and_then(Value::as_str) {
            if !matches!(kind, "game" | "lobby" | "game_info" | "game_created") {
                return None;
            }
        }

        let id = value
            .get("id")
            .or_else(|| value.get("game_id"))
            .or_else(|| value.get("lobby_id"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let name = value
            .get("name")
            .or_else(|| value.get("title"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| {
                if id == 0 {
                    "Open game".to_owned()
                } else {
                    format!("Game {id}")
                }
            });

        let looks_like_lobby = value.get("name").is_some()
            || value.get("title").is_some()
            || value.get("id").is_some()
            || value.get("game_id").is_some()
            || value.get("lobby_id").is_some();
        if !looks_like_lobby {
            return None;
        }

        Some(Self {
            id,
            name,
            game: value
                .get("game")
                .and_then(Value::as_str)
                .unwrap_or(DEFAULT_GAME)
                .to_owned(),
            players: player_count(value),
            max_players: number_field(value, &["max_players", "capacity", "slots"]),
            status: value
                .get("status")
                .or_else(|| value.get("state"))
                .or_else(|| value.get("phase"))
                .and_then(Value::as_str)
                .unwrap_or("open")
                .to_owned(),
        })
    }
}

fn player_count(value: &Value) -> u32 {
    if let Some(players) = value.get("players") {
        if let Some(items) = players.as_array() {
            return items.len() as u32;
        }
        if let Some(count) = players.as_u64() {
            return count as u32;
        }
    }

    number_field(
        value,
        &[
            "player_count",
            "num_players",
            "current_players",
            "players_count",
        ],
    )
}

fn number_field(value: &Value, keys: &[&str]) -> u32 {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_u64))
        .unwrap_or(0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_json_command_as_one_line() {
        let payload = json!({
            "type": "create_game",
            "name": "Tactics lobby",
            "game": "tactics",
            "max_players": 4
        });
        let mut output = Vec::new();

        write_json_line(&mut output, &payload).expect("json command should write");

        assert_eq!(
            String::from_utf8(output).expect("command should be utf8"),
            "{\"game\":\"tactics\",\"max_players\":4,\"name\":\"Tactics lobby\",\"type\":\"create_game\"}\n"
        );
    }

    #[test]
    fn parses_game_created_nested_game_as_lobby() {
        let payload = json!({
            "type": "game_created",
            "game": {
                "id": 1,
                "name": "Friday lobby",
                "game": "tactics",
                "host_id": 1,
                "max_players": 4,
                "status": "lobby",
                "players": [
                    {
                        "id": 1,
                        "name": "Ada",
                        "ping_ms": 12,
                        "latency_ms": 6,
                        "game_id": 1
                    }
                ]
            }
        });

        let lobbies = lobbies_from_value(&payload).expect("game_created should parse");
        assert_eq!(
            lobbies,
            vec![Lobby {
                id: 1,
                name: "Friday lobby".to_owned(),
                game: "tactics".to_owned(),
                players: 1,
                max_players: 4,
                status: "lobby".to_owned(),
            }]
        );
    }
}

#[allow(dead_code)]
fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect(DEFAULT_SERVER_ADDR)?;
    stream.set_nodelay(true)?;

    let session_id = lobby_session_id();
    let started = Instant::now();
    send(
        &mut stream,
        json!({
            "type": "hello",
            "name": "Ada",
            "ping_ms": 0,
            "latency_ms": 0,
            "game": DEFAULT_GAME,
            "session_id": session_id
        }),
    )?;
    send(&mut stream, json!({"type": "game_list"}))?;
    send(
        &mut stream,
        json!({
            "type": "create_game",
            "name": "Friday lobby",
            "game": DEFAULT_GAME,
            "max_players": 4,
            "session_id": session_id
        }),
    )?;
    send(&mut stream, json!({"type": "start_game", "game_id": 1, "session_id": session_id}))?;

    let mut read = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    loop {
        if started.elapsed() > Duration::from_secs(1) {
            send(&mut stream, json!({"type": "heartbeat", "ping_ms": 12, "latency_ms": 6}))?;
            send(
                &mut stream,
                json!({
                    "type": "position",
                    "game_id": 1,
                    "state": {"x": 12.0, "y": 4.0, "facing": "east"}
                }),
            )?;
        }

        line.clear();
        if read.read_line(&mut line)? == 0 {
            break;
        }
        println!("server: {}", line.trim_end());
    }
    Ok(())
}

/*
Useful commands:
{"type":"chat","text":"hello"}
{"type":"join_game","game_id":1}
{"type":"pause_game","game_id":1,"session_id":"same-as-create"}
{"type":"resume_game","game_id":1,"session_id":"same-as-create"}
{"type":"game_command","game_id":1,"seq":42,"command":{"move":{"dx":1,"dy":0}}}
{"type":"free_game","game_id":1,"session_id":"same-as-create"}
*/
