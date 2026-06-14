use serde::{Deserialize, Serialize, Serializer};
use std::{error::Error, fmt, fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentData {
    pub path: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct DocumentError {
    pub code: &'static str,
    pub message: String,
}

impl DocumentError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl Serialize for DocumentError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct ErrorPayload<'a> {
            code: &'a str,
            message: &'a str,
        }

        ErrorPayload {
            code: self.code,
            message: &self.message,
        }
        .serialize(serializer)
    }
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error for DocumentError {}

fn document_title(path: &PathBuf) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("未命名")
        .to_string()
}

fn read_document(path: PathBuf) -> Result<DocumentData, DocumentError> {
    let content = fs::read_to_string(&path).map_err(|error| {
        DocumentError::new(
            "read failed",
            format!("failed to read document {} as UTF-8: {error}", path.display()),
        )
    })?;

    Ok(DocumentData {
        title: document_title(&path),
        path: path.to_string_lossy().into_owned(),
        content,
    })
}

fn write_document(path: PathBuf, content: &str) -> Result<DocumentData, DocumentError> {
    fs::write(&path, content.as_bytes()).map_err(|error| {
        DocumentError::new(
            "write failed",
            format!("failed to write document {} as UTF-8: {error}", path.display()),
        )
    })?;

    Ok(DocumentData {
        title: document_title(&path),
        path: path.to_string_lossy().into_owned(),
        content: content.to_string(),
    })
}

#[tauri::command]
pub fn open_document() -> Result<Option<DocumentData>, DocumentError> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Markdown", &["md", "markdown", "mdown"])
        .add_filter("Text", &["txt"])
        .add_filter("All files", &["*"])
        .pick_file()
    else {
        return Ok(None);
    };

    read_document(path).map(Some)
}

#[tauri::command]
pub fn save_document(path: String, content: String) -> Result<DocumentData, DocumentError> {
    write_document(PathBuf::from(path), &content)
}

#[tauri::command]
pub fn save_document_as(content: String) -> Result<Option<DocumentData>, DocumentError> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Markdown", &["md", "markdown", "mdown"])
        .add_filter("Text", &["txt"])
        .set_file_name("未命名.md")
        .save_file()
    else {
        return Ok(None);
    };

    write_document(path, &content).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    fn unique_test_file(name: &str) -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir()
            .join(format!("notedown-document-tests-{id}"))
            .join(name)
    }

    #[test]
    fn write_and_read_document_round_trips_utf8_content() {
        let path = unique_test_file("测试.md");
        let content = "# 标题\n\n正文";
        fs::create_dir_all(path.parent().expect("expected parent")).expect("expected test dir setup");

        let saved = write_document(path.clone(), content).expect("expected write to succeed");
        let opened = read_document(path.clone()).expect("expected read to succeed");

        assert_eq!(saved.title, "测试.md");
        assert_eq!(opened.content, content);

        fs::remove_file(&path).expect("expected test file cleanup");
        fs::remove_dir(path.parent().expect("expected parent")).expect("expected test dir cleanup");
    }

    #[test]
    fn read_missing_document_returns_read_error() {
        let path = unique_test_file("missing.md");

        let error = read_document(path).expect_err("expected read failure");

        assert_eq!(error.code, "read failed");
    }
}
