use cucumber::{given, then, when, World};
use thirtyfour::{By, DesiredCapabilities, WebDriver};

#[derive(Debug, Default, World)]
pub struct UiWorld {
    pub base_url: String,
    pub email: Option<String>,
    pub password: Option<String>,
    pub driver: Option<WebDriver>,
}

#[given(regex = r#"^the app is running at "([^"]+)"$"#)]
async fn app_up(w: &mut UiWorld, url: String) {
    w.base_url = url;
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless=new").expect("caps");
    caps.add_arg("--window-size=1280,720").expect("caps");
    w.driver = Some(
        WebDriver::new("http://localhost:4444", caps)
            .await
            .expect("could not start WebDriver — is chromedriver running on :4444?"),
    );
}

#[given(regex = r#"^a registered user "([^"]+)" with password "([^"]+)"$"#)]
async fn user(w: &mut UiWorld, email: String, password: String) {
    w.email = Some(email);
    w.password = Some(password);
}

#[when(regex = r"^they navigate to the sign-in page$")]
async fn goto_signin(w: &mut UiWorld) {
    let driver = w.driver.as_ref().expect("driver not initialized");
    driver
        .goto(format!("{}/sign-in", w.base_url))
        .await
        .expect("navigation failed");
    driver
        .find(By::Css("[data-testid='sign-in-form']"))
        .await
        .expect("sign-in form did not render");
}

#[when(regex = r"^they fill the sign-in form with those credentials$")]
async fn fill_creds(w: &mut UiWorld) {
    fill(w, w.email.clone().unwrap_or_default(), w.password.clone().unwrap_or_default()).await;
}

#[when(regex = r#"^they fill the sign-in form with password "([^"]+)"$"#)]
async fn fill_bad_password(w: &mut UiWorld, password: String) {
    fill(w, w.email.clone().unwrap_or_default(), password).await;
}

async fn fill(w: &UiWorld, email: String, password: String) {
    let driver = w.driver.as_ref().unwrap();
    driver
        .find(By::Css("[data-testid='email-input']"))
        .await
        .unwrap()
        .send_keys(email)
        .await
        .unwrap();
    driver
        .find(By::Css("[data-testid='password-input']"))
        .await
        .unwrap()
        .send_keys(password)
        .await
        .unwrap();
}

#[when(regex = r"^they submit the form$")]
async fn submit(w: &mut UiWorld) {
    let driver = w.driver.as_ref().unwrap();
    driver
        .find(By::Css("[data-testid='submit-button']"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();
}

#[then(regex = r"^they land on the dashboard$")]
async fn on_dashboard(w: &mut UiWorld) {
    let driver = w.driver.as_ref().unwrap();
    let url = driver.current_url().await.unwrap();
    assert!(url.as_str().contains("/dashboard"), "url = {url}");
}

#[then(regex = r"^they remain on the sign-in page$")]
async fn on_signin(w: &mut UiWorld) {
    let driver = w.driver.as_ref().unwrap();
    let url = driver.current_url().await.unwrap();
    assert!(url.as_str().contains("/sign-in"), "url = {url}");
}

#[then(regex = r#"^the form shows the error "([^"]+)"$"#)]
async fn shows_error(w: &mut UiWorld, expected: String) {
    let driver = w.driver.as_ref().unwrap();
    let text = driver
        .find(By::Css("[data-testid='form-error']"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(text, expected);
}

#[tokio::main]
async fn main() {
    UiWorld::cucumber()
        .after(|_, _, _, _, w| {
            Box::pin(async move {
                if let Some(driver) = w.and_then(|w| w.driver.take()) {
                    let _ = driver.quit().await;
                }
            })
        })
        .run("tests/features")
        .await;
}
