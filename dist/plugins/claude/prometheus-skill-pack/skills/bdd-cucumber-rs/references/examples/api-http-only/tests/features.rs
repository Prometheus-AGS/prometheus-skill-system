use cucumber::{given, then, when, World};
use serde_json::Value;

#[derive(Debug, Default, World)]
pub struct AuthWorld {
    pub base_url: String,
    pub email: Option<String>,
    pub password: Option<String>,
    pub status: Option<u16>,
    pub body: Option<Value>,
}

#[given(regex = r#"^the auth service is reachable at "([^"]+)"$"#)]
async fn service_up(w: &mut AuthWorld, url: String) {
    w.base_url = url;
}

#[given(regex = r#"^a registered user "([^"]+)" with password "([^"]+)"$"#)]
async fn user(w: &mut AuthWorld, email: String, password: String) {
    w.email = Some(email);
    w.password = Some(password);
}

#[when(regex = r#"^they POST to "([^"]+)" with those credentials$"#)]
async fn post_creds(w: &mut AuthWorld, path: String) {
    let url = format!("{}{}", w.base_url, path);
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({
            "email": w.email,
            "password": w.password,
        }))
        .send()
        .await
        .expect("request failed");
    w.status = Some(resp.status().as_u16());
    w.body = Some(resp.json().await.expect("bad json"));
}

#[when(regex = r#"^they POST to "([^"]+)" with password "([^"]+)"$"#)]
async fn post_bad_password(w: &mut AuthWorld, path: String, password: String) {
    let url = format!("{}{}", w.base_url, path);
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({
            "email": w.email,
            "password": password,
        }))
        .send()
        .await
        .expect("request failed");
    w.status = Some(resp.status().as_u16());
    w.body = Some(resp.json().await.expect("bad json"));
}

#[then(regex = r"^the response status is (\d+)$")]
async fn status_is(w: &mut AuthWorld, expected: u16) {
    assert_eq!(w.status, Some(expected));
}

#[then(regex = r#"^the response body contains a non-empty "([^"]+)" field$"#)]
async fn field_present(w: &mut AuthWorld, field: String) {
    let val = w.body.as_ref().and_then(|b| b.get(&field));
    assert!(
        val.and_then(Value::as_str).map(str::is_empty) == Some(false),
        "{field} missing or empty"
    );
}

#[then(regex = r#"^the response body contains error message "([^"]+)"$"#)]
async fn error_message(w: &mut AuthWorld, expected: String) {
    let msg = w
        .body
        .as_ref()
        .and_then(|b| b.get("error"))
        .and_then(Value::as_str)
        .unwrap_or("");
    assert_eq!(msg, expected);
}

#[tokio::main]
async fn main() {
    AuthWorld::run("tests/features").await;
}
