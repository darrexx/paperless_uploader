use std::{fs, path::PathBuf, time::Duration};

use reqwest::{
    blocking::multipart::{Form, Part},
    IntoUrl,
};

use crate::error_logging::LogError;

pub fn process_new_document<U>(
    path: PathBuf,
    url: U,
    token: String,
) -> Result<reqwest::blocking::Response, anyhow::Error>
where
    U: IntoUrl,
{
    std::thread::sleep(Duration::from_secs(60));

    let url = url.into_url().log_if_error("given url not a valid url")?;
    let posturl = url
        .join("/api/documents/post_document/")
        .log_if_error("joining api endpoint failed")?;

    let client = reqwest::blocking::Client::new(); //Todo change to non blocking

    let body = build_body(&path)?;

    let res = client
        .post(posturl)
        .header("Authorization", format!("Token {}", token))
        .multipart(body)
        .send()
        .log_if_error("Sending Post Request to paperless ngx failed")?;

    let res = res
        .error_for_status()
        .log_if_error("Paperless return failure Status Code")?;

    log::info!("Successfully uploaded Document! {}", res.status());

    fs::remove_file(path).log_if_error("Error deleting uploaded Document")?;

    Ok(res)
}

fn build_body(path: &PathBuf) -> Result<Form, anyhow::Error> {
    let file_part =
        Part::file(path).log_if_error("Could not create file part of fiel to be uploaded")?;
    let form = Form::new();
    Ok(form.part("document", file_part))
}
