use std::path::PathBuf;

use axum::response::Html;

pub fn get_mime(file_path: &PathBuf) -> Result<&str, Box<dyn std::error::Error>> {
    let mime = infer::get_from_path(file_path)?;
    match mime {
        Some(mime) => Ok(mime.mime_type()),
        None => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not determine MIME type",
        ))),
    }
}

pub fn err_template(code: u32) -> Html<String> {
    let (title, msg) = match code {
        301 => ("301 Moved Permanently", "The page has been moved."),
        302 => ("302 Found", "The page has been moved temporarily."),
        400 => ("400 Bad Request", "The request is invalid."),
        404 => (
            "404 Not Found",
            "The page you are looking for does not exist.",
        ),
        401 => (
            "401 Unauthorized",
            "You are not authorized to view this page.",
        ),
        _ => (
            "500 Internal Server Error",
            "An error occurred while processing your request.",
        ),
    };

    let template = format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Error {}</title>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    text-align: center;
                    margin: 50px;
                }}
            </style>
        </head>
        <body>
            <h1>Error {}</h1>
            <p>{}</p>
        </body>
        </html>"#,
        title, title, msg
    );

    Html(template)
}
