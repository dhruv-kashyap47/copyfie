//! Thumbnail delivery over the `copyfie://` custom protocol (stub).
//!
//! Images themselves are M2-04. This handler exists from day one so the
//! palette can render thumbnails without base64 over IPC, and so the URL shape,
//! validation, and caching headers are settled before the image pipeline lands.
//!
//! URL forms:
//! - macOS/Linux: `copyfie://localhost/<64-hex>.png`
//! - Windows:     `http://copyfie.localhost/<64-hex>.png`
//!
//! Security:
//! - Only 64 lowercase hex characters are accepted; nothing else reaches the
//!   filesystem, so path traversal is impossible.
//! - Files are content-addressed, so responses are immutable for a year.

use std::path::{Path, PathBuf};

use tauri::http::{header, Request, Response, StatusCode};

/// Directory under the app data root that holds `<hash>.png` thumbnails.
pub const THUMBS_DIR: &str = "thumbs";

const CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

/// True when `name` is exactly 64 lowercase hexadecimal characters.
pub fn is_valid_key(name: &str) -> bool {
    name.len() == 64
        && name
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Resolve a validated key to a thumbnail path.
pub fn thumb_path(thumbs_dir: &Path, key: &str) -> Option<PathBuf> {
    is_valid_key(key).then(|| thumbs_dir.join(format!("{key}.png")))
}

/// Serve one thumbnail request.
pub fn serve(thumbs_dir: &Path, request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let raw = request.uri().path().trim_start_matches('/');
    let key = raw.strip_suffix(".png").unwrap_or(raw);

    let Some(path) = thumb_path(thumbs_dir, key) else {
        return error(StatusCode::BAD_REQUEST, "invalid thumbnail key");
    };

    match std::fs::read(&path) {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/png")
            .header(header::CACHE_CONTROL, CACHE_CONTROL)
            .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
            .body(bytes)
            .unwrap_or_else(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "response build failed")),
        Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => {
            error(StatusCode::NOT_FOUND, "thumbnail not found")
        }
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "thumbnail read failed"),
    }
}

/// Response used when the app data directory cannot be resolved.
pub fn unavailable() -> Response<Vec<u8>> {
    error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "app data directory unavailable",
    )
}

fn error(status: StatusCode, message: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .body(message.as_bytes().to_vec())
        .expect("static error response is valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(uri: &str) -> Request<Vec<u8>> {
        Request::builder()
            .uri(uri)
            .body(Vec::new())
            .expect("request")
    }

    fn key() -> String {
        "0123456789abcdef".repeat(4)
    }

    #[test]
    fn validates_keys_strictly() {
        assert!(is_valid_key(&key()));
        assert!(!is_valid_key(&key().to_uppercase()));
        assert!(!is_valid_key(&key()[..63]));
        assert!(!is_valid_key(&format!("{}0", key())));
        assert!(!is_valid_key("../etc/passwd"));
        assert!(!is_valid_key("..%2F.."));
        assert!(!is_valid_key(""));
        assert!(!is_valid_key("0123456789abcdeg".repeat(4).as_str()));
    }

    #[test]
    fn path_is_only_built_for_valid_keys() {
        let dir = Path::new("C:\\data\\thumbs");
        assert_eq!(
            thumb_path(dir, &key()),
            Some(dir.join(format!("{}.png", key())))
        );
        assert_eq!(thumb_path(dir, "../secrets"), None);
    }

    #[test]
    fn serves_existing_thumbnail_with_immutable_cache() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bytes = b"\x89PNG\r\n\x1a\nfake";
        std::fs::write(temp.path().join(format!("{}.png", key())), bytes).expect("write");

        let response = serve(
            temp.path(),
            &request(&format!("copyfie://localhost/{}.png", key())),
        );
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).expect("ct"),
            "image/png"
        );
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).expect("cc"),
            CACHE_CONTROL
        );
        assert_eq!(
            response
                .headers()
                .get(header::X_CONTENT_TYPE_OPTIONS)
                .expect("nosniff"),
            "nosniff"
        );
        assert_eq!(response.body(), bytes);
    }

    #[test]
    fn accepts_the_windows_url_form() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join(format!("{}.png", key())), b"png").expect("write");

        let response = serve(
            temp.path(),
            &request(&format!("http://copyfie.localhost/{}", key())),
        );
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn missing_thumbnail_is_not_found() {
        let temp = tempfile::tempdir().expect("tempdir");
        let response = serve(
            temp.path(),
            &request(&format!("copyfie://localhost/{}", key())),
        );
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn missing_directory_is_not_found() {
        let temp = tempfile::tempdir().expect("tempdir");
        let response = serve(
            &temp.path().join("does-not-exist"),
            &request(&format!("copyfie://localhost/{}", key())),
        );
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn rejects_invalid_keys_before_touching_the_filesystem() {
        let temp = tempfile::tempdir().expect("tempdir");
        let response = serve(temp.path(), &request("copyfie://localhost/../secrets.png"));
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).expect("cc"),
            "no-store"
        );
    }
}
