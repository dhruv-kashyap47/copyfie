//! Foreground app identity on X11.
//!
//! One short-lived X connection per capture. Captures are infrequent and this
//! runs only after the privacy gate, so the ~1 ms connect/handshake cost is
//! preferable to keeping a non-`Sync` connection in shared state.

use copyfie_core::SourceApp;
use copyfie_platform::ForegroundApp;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;

pub struct X11Foreground;

impl ForegroundApp for X11Foreground {
    fn current(&self) -> Option<SourceApp> {
        current().ok().flatten()
    }
}

fn current() -> Result<Option<SourceApp>, Box<dyn std::error::Error>> {
    let (connection, screen_number) = x11rb::connect(None)?;
    let screen = connection
        .setup()
        .roots
        .get(screen_number)
        .ok_or("X11 screen not found")?;
    let root = screen.root;

    let active_atom = intern(&connection, "_NET_ACTIVE_WINDOW")?;
    let pid_atom = intern(&connection, "_NET_WM_PID")?;
    let class_atom = intern(&connection, "WM_CLASS")?;
    let name_atom = intern(&connection, "_NET_WM_NAME")?;
    let utf8_atom = intern(&connection, "UTF8_STRING")?;

    let Some(active) = first_u32(&connection, root, active_atom, AtomEnum::WINDOW)? else {
        return Ok(None);
    };
    if active == 0 || active == root {
        return Ok(None);
    }

    let pid = first_u32(&connection, active, pid_atom, AtomEnum::CARDINAL)?;
    let comm = pid.and_then(process_comm);
    let class = wm_class(&connection, active, class_atom);
    let title = window_title(&connection, active, name_atom, utf8_atom);

    let id = class.clone().or_else(|| comm.clone());
    let name = comm.or(class);

    if id.is_none() && name.is_none() && title.is_none() {
        return Ok(None);
    }

    Ok(Some(SourceApp {
        id,
        name,
        window_title: title,
    }))
}

fn intern(connection: &RustConnection, name: &str) -> Result<Atom, Box<dyn std::error::Error>> {
    Ok(connection
        .intern_atom(false, name.as_bytes())?
        .reply()?
        .atom)
}

fn first_u32(
    connection: &RustConnection,
    window: Window,
    property: Atom,
    kind: AtomEnum,
) -> Result<Option<u32>, Box<dyn std::error::Error>> {
    let reply = connection
        .get_property(false, window, property, kind, 0, 1)?
        .reply()?;

    Ok(reply.value32().and_then(|mut values| values.next()))
}

fn wm_class(connection: &RustConnection, window: Window, property: Atom) -> Option<String> {
    let reply = connection
        .get_property(false, window, property, AtomEnum::STRING, 0, 256)
        .ok()?
        .reply()
        .ok()?;

    parse_wm_class(&reply.value)
}

fn window_title(
    connection: &RustConnection,
    window: Window,
    property: Atom,
    kind: Atom,
) -> Option<String> {
    let reply = connection
        .get_property(false, window, property, kind, 0, 1024)
        .ok()?
        .reply()
        .ok()?;

    let title = String::from_utf8_lossy(&reply.value).trim().to_string();
    (!title.is_empty()).then_some(title)
}

/// `WM_CLASS` is two NUL-terminated strings: instance then class. Prefer the
/// class, which is the stable, capitalized identity (`KeePassXC`).
fn parse_wm_class(value: &[u8]) -> Option<String> {
    let mut parts = value
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty());
    let instance = parts.next()?;
    let class = parts.next().unwrap_or(instance);
    let class = String::from_utf8_lossy(class).trim().to_string();
    (!class.is_empty()).then_some(class)
}

fn process_comm(pid: u32) -> Option<String> {
    let comm = std::fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
    let comm = comm.trim().to_string();
    (!comm.is_empty()).then_some(comm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wm_class_instance_and_class() {
        assert_eq!(
            parse_wm_class(b"keepassxc\0KeePassXC\0").as_deref(),
            Some("KeePassXC")
        );
        assert_eq!(parse_wm_class(b"Navigator\0").as_deref(), Some("Navigator"));
        assert_eq!(parse_wm_class(b""), None);
        assert_eq!(parse_wm_class(b"\0\0"), None);
    }
}
