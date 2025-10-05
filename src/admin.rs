use crate::config::{Action, DynamicSettings, Persisted, Profile};
use crate::{AdminPasswordValidator, ArmQRState};
use askama::Template;
use axum::extract::{FromRef, FromRequestParts, Query, State};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
use axum_auth::AuthBasic;
use http::StatusCode;
use http::uri::PathAndQuery;
use query_string_builder::{QueryString, QueryStringSimple};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminPage<'a> {
    pub config: &'a DynamicSettings,
    pub error: Option<&'a str>,
}

#[derive(Debug)]
pub struct AdminUser {
    _priv: (),
}

impl AdminUser {
    pub unsafe fn assert() -> Self {
        Self { _priv: () }
    }
}

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync + AdminPasswordValidator,
{
    type Rejection = http::Response<String>;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            match AuthBasic::from_request_parts(parts, state).await {
                Ok(AuthBasic((user, Some(password))))
                    if user == "admin"
                        && state.validate_admin_password(password.clone().into()) =>
                {
                    Ok(unsafe { Self::assert() })
                }
                _ => Err(http::Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("WWW-Authenticate", r#"Basic realm="Dev", charset="UTF-8""#)
                    .body("".into())
                    .unwrap()),
            }
        }
    }
}

pub fn admin_subrouter<S>() -> Router<S>
where
    S: AdminPasswordValidator + Clone + Send + Sync + 'static,
    Persisted<DynamicSettings>: FromRef<S>,
{
    let router = Router::new()
        .route("/", get(admin_page))
        .route("/profiles", post(create_profile_form))
        .route("/activateProfile", post(activate_profile_form))
        .route("/deleteProfile", post(delete_profile_form));
    router
}

#[axum::debug_handler(state = ArmQRState)]
async fn admin_page(
    _admin: AdminUser,
    Query(params): Query<HashMap<String, String>>,
    State(settings): State<Persisted<DynamicSettings>>,
) -> Html<String> {
    let page = {
        let config = settings.snapshot().await;
        AdminPage {
            config: &config,
            error: params.get("error").map(|x| x.as_str()),
        }
        .render()
        .unwrap()
    };
    Html(page)
}

#[derive(Deserialize)]
struct NewProfileForm {
    name: Option<String>,
    redirect_uri: String,
}

#[axum::debug_handler(state = ArmQRState)]
async fn create_profile_form(
    _admin: AdminUser,
    State(settings): State<Persisted<DynamicSettings>>,
    Form(form): Form<NewProfileForm>,
) -> Redirect {
    if let Err(e) = validate_url(&form.redirect_uri) {
        return Redirect::to(&format!(
            "/admin{}",
            QueryString::dynamic().with_value("error", e.to_string())
        ));
    };

    let name = match form.name {
        Some(x) => x.to_string(),
        None => format!("Redirect: {}", form.redirect_uri),
    };
    let id = Uuid::new_v4();

    let mut config = settings.snapshot().await.as_ref().clone();
    config.profiles.insert(
        id,
        Profile {
            name,
            action: Action::Redirect(form.redirect_uri.to_string()),
        },
    );

    settings.store(config).await;

    Redirect::to("/admin")
}

fn validate_url(url: &str) -> Result<(), Box<dyn Error>> {
    if url.is_empty() {
        Err("no URL provided")?
    }

    let url = url.parse::<http::Uri>()?;

    if url.scheme().is_none() {
        Err("missing scheme")?
    }

    if url.host().is_none() {
        Err("missing host")?
    }

    Ok(())
}

#[derive(Deserialize)]
struct ActivateProfileForm {
    id: String,
}

#[axum::debug_handler(state = ArmQRState)]
async fn activate_profile_form(
    _admin: AdminUser,
    State(settings): State<Persisted<DynamicSettings>>,
    Form(form): Form<ActivateProfileForm>,
) -> Redirect {
    let uuid = match Uuid::from_str(&form.id) {
        Ok(uuid) => uuid,
        Err(_) => return Redirect::to("/admin?error=bad_uuid"),
    };

    let mut config = settings.snapshot_cloned().await;

    if !config.profiles.contains_key(&uuid) {
        return Redirect::to("/admin?error=bad_uuid");
    }

    config.current_profile_id = uuid;

    settings.store(config).await;

    Redirect::to("/admin")
}

#[derive(Deserialize)]
struct DeleteProfileForm {
    id: String,
}

#[axum::debug_handler(state = ArmQRState)]
async fn delete_profile_form(
    _admin: AdminUser,
    State(settings): State<Persisted<DynamicSettings>>,
    Form(form): Form<DeleteProfileForm>,
) -> Redirect {
    let uuid = match Uuid::from_str(&form.id) {
        Ok(uuid) => uuid,
        Err(_) => return Redirect::to("/admin?error=bad_uuid"),
    };

    let mut config = settings.snapshot_cloned().await;

    config.profiles.remove(&uuid);

    settings.store(config).await;

    Redirect::to("/admin")
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("https://google.com")]
    #[case("https://astrid.tech")]
    #[case("http://astrid.tech")]
    #[case("ftp://some/server")]
    fn test_validate_url_is_successful(#[case] input: &str) {
        validate_url(input).expect("does not validate");
    }

    #[rstest]
    #[case("")]
    #[case("foo")]
    #[case("bar")]
    #[case("astrid.tech")]
    #[case("bar/spam")]
    fn test_validate_url_fails(#[case] input: &str) {
        validate_url(input).expect_err("wrongly validates");
    }
}
