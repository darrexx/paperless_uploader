use std::{fs, path::PathBuf, time::Duration};

use anyhow::anyhow;
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

    let body = build_body(path)?;

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
    Ok(res)
}

fn build_body(path: PathBuf) -> Result<Form, anyhow::Error> {
    let content = fs::read(&path).log_if_error("Could not read file to be uploaded")?;

    let filename = path
        .file_name()
        .ok_or_else(|| anyhow!("could not read filename of path"))
        .log_if_error(format!("could not read filename of path {:?}", path))?
        .to_os_string()
        .into_string()
        .map_err(|_| anyhow!("Could not conver OsString to String"))
        .log_if_error("Could not conver OsString to String")?;

    let file_part = Part::bytes(content).file_name(filename); //change to use stream
    let form = Form::new();
    Ok(form.part("document", file_part))
}
