//! Page routing, errors, and data structures.

use crate::data::AppDatabase;
use crate::service;
use crate::service::action;
use crate::web::{
    ctx, form, renderer::Renderer, responsecounter::ResponseCounter, PageError, PASSWORD_COOKIE,
};
use crate::{ServiceError, ShortCode};
use rocket::form::{Contextual, Form};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::response::content::RawHtml;
use rocket::response::{status, Redirect};
use rocket::{uri, State};

/// Route to the home page.
#[rocket::get("/")]
fn home(renderer: &State<Renderer<'_>>) -> RawHtml<String> {
    let context = ctx::Home::default();
    RawHtml(renderer.render(context, &[]))
}

/// Route to submit a new [`Job`](crate::Job).
#[rocket::post("/", data = "<form>")]
pub async fn new_job(
    form: Form<Contextual<'_, form::NewJob>>,
    database: &State<AppDatabase>,
    renderer: &State<Renderer<'_>>,
) -> Result<Redirect, (Status, RawHtml<String>)> {
    let form = form.into_inner();
    if let Some(value) = form.value {
        let req = service::ask::NewJob {
            escrow_id: value.escrow_id,
            manifest_id: value.manifest_id,
            posted: value.posted,
            expires: value.expires,
            password: value.password,
        };
        match action::new_job(req, database.get_pool()).await {
            Ok(job) => Ok(Redirect::to(uri!(get_job(shortcode = job.shortcode)))),
            Err(e) => {
                eprintln!("internal error: {}", e);
                Err((
                    Status::InternalServerError,
                    RawHtml(renderer.render(
                        ctx::Home::default(),
                        &["A server error occurred. Please try again"],
                    )),
                ))
            }
        }
    } else {
        let errors = form
            .context
            .errors()
            .map(|err| {
                use rocket::form::error::ErrorKind;
                if let ErrorKind::Validation(msg) = &err.kind {
                    msg.as_ref()
                } else {
                    eprintln!("unhandled error: {}", err);
                    "An error occurred, please try again"
                }
            })
            .collect::<Vec<_>>();
        Err((
            Status::BadRequest,
            RawHtml(renderer.render_with_data(
                ctx::Home::default(),
                ("job", &form.context),
                &errors,
            )),
        ))
    }
}

/// Route to get a [`Job`](crate::Job).
#[rocket::get("/job/<shortcode>")]
pub async fn get_job(
    shortcode: ShortCode,
    database: &State<AppDatabase>,
    hit_counter: &State<ResponseCounter>,
    renderer: &State<Renderer<'_>>,
) -> Result<status::Custom<RawHtml<String>>, PageError> {
    fn render_with_status<T: ctx::PageContext + serde::Serialize + std::fmt::Debug>(
        status: Status,
        context: T,
        renderer: &Renderer,
    ) -> Result<status::Custom<RawHtml<String>>, PageError> {
        Ok(status::Custom(
            status,
            RawHtml(renderer.render(context, &[])),
        ))
    }
    match action::get_job(shortcode.clone().into(), database.get_pool()).await {
        Ok(job) => {
            hit_counter.hit(shortcode.clone(), 1);
            let context = ctx::ViewJob::new(job);
            render_with_status(Status::Ok, context, renderer)
        }
        Err(e) => match e {
            ServiceError::PermissionError(_) => {
                let context = ctx::PasswordRequired::new(shortcode);
                render_with_status(Status::Unauthorized, context, renderer)
            }
            ServiceError::NotFound => Err(PageError::NotFound("Job not found".to_owned())),
            _ => Err(PageError::Internal("server error".to_owned())),
        },
    }
}

/// Route to submit a [`Password`](crate::domain::job::field::Password) for a password-protected [`Job`](crate::Job).
#[rocket::post("/job/<shortcode>", data = "<form>")]
pub async fn submit_job_password(
    cookies: &CookieJar<'_>,
    form: Form<Contextual<'_, form::GetPasswordProtectedJob>>,
    shortcode: ShortCode,
    hit_counter: &State<ResponseCounter>,
    database: &State<AppDatabase>,
    renderer: &State<Renderer<'_>>,
) -> Result<RawHtml<String>, PageError> {
    if let Some(form) = &form.value {
        let req = service::ask::GetJob {
            shortcode: shortcode.clone(),
            password: form.password.clone(),
        };
        match action::get_job(req, database.get_pool()).await {
            Ok(job) => {
                hit_counter.hit(shortcode.clone(), 1);
                let context = ctx::ViewJob::new(job);
                cookies.add(Cookie::new(
                    PASSWORD_COOKIE,
                    form.password.clone().into_inner().unwrap_or_default(),
                ));
                Ok(RawHtml(renderer.render(context, &[])))
            }
            Err(e) => match e {
                ServiceError::PermissionError(e) => {
                    let context = ctx::PasswordRequired::new(shortcode);
                    Ok(RawHtml(renderer.render(context, &[e.as_str()])))
                }
                ServiceError::NotFound => Err(PageError::NotFound("Job not found".to_owned())),
                _ => Err(PageError::Internal("server error".to_owned())),
            },
        }
    } else {
        let context = ctx::PasswordRequired::new(shortcode);
        Ok(RawHtml(renderer.render(
            context,
            &["A password is required to view this job"],
        )))
    }
}

/// Route to get just the [`EscrowId`](crate::domain::job::field::EscrowId) of a [`Job`](crate::Job).
#[rocket::get("/job/raw/<shortcode>")]
pub async fn get_raw_job(
    cookies: &CookieJar<'_>,
    shortcode: ShortCode,
    hit_counter: &State<ResponseCounter>,
    database: &State<AppDatabase>,
) -> Result<status::Custom<String>, Status> {
    use crate::domain::job::field::Password;
    let req = service::ask::GetJob {
        shortcode: shortcode.clone(),
        password: cookies
            .get(PASSWORD_COOKIE)
            .map(|cookie| cookie.value())
            .map(|raw_password| Password::new(raw_password.to_string()).ok())
            .flatten()
            .unwrap_or_else(Password::default),
    };
    match action::get_job(req, database.get_pool()).await {
        Ok(job) => {
            hit_counter.hit(shortcode.clone(), 1);
            Ok(status::Custom(Status::Ok, job.escrow_id.into_inner()))
        }
        Err(e) => match e {
            ServiceError::PermissionError(msg) => Ok(status::Custom(Status::Unauthorized, msg)),
            ServiceError::NotFound => Err(Status::NotFound),
            _ => Err(Status::InternalServerError),
        },
    }
}

/// The URI [`routes`](rocket::Route) which can be mounted by [`rocket`].
pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![home, get_job, new_job, submit_job_password, get_raw_job]
}

pub mod catcher {
    //! Contains all the page catchers.
    use rocket::Request;
    use rocket::{catch, catchers, Catcher};

    /// Catch unhandled errors.
    #[catch(default)]
    fn default(req: &Request) -> &'static str {
        eprintln!("General error: {:?}", req);
        "something went wrong..."
    }

    /// Catch server errors.
    #[catch(500)]
    fn internal_error(req: &Request) -> &'static str {
        eprintln!("Internal error: {:?}", req);
        "internal server error"
    }

    /// Catch missing data errors.
    #[catch(404)]
    fn not_found() -> &'static str {
        "404"
    }

    /// The [`catchers`](rocket::Catcher) which can be registered by [`rocket`].
    pub fn catchers() -> Vec<Catcher> {
        catchers![not_found, default, internal_error]
    }
}

#[cfg(test)]
pub mod test {
    use crate::{data::AppDatabase, web::test::init_test_client};
    use rocket::http::Status;

    #[test]
    fn gets_home() {
        let (_, client) = init_test_client();

        let response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn error_on_missing_job() {
        let (_, client) = init_test_client();

        let response = client.get("/job/aasldfjkasldgkj").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn requires_password_when_applicable() {
        use crate::domain::job::field::{EscrowId, Expires, ManifestId, Password};
        use crate::service;
        use rocket::http::{Cookie, EscrowIdType};

        let (rt, client) = init_test_client();

        let db = client.rocket().state::<AppDatabase>().unwrap();

        let req = service::ask::NewJob {
            escrow_id: EscrowId::new("escrow_id").unwrap(),
            expires: Expires::default(),
            password: Password::new("123".to_owned()).unwrap(),
            title: ManifestId::default(),
        };
        let job = rt
            .block_on(async move { service::action::new_job(req, db.get_pool()).await })
            .unwrap();

        // Block job when no password is provided
        let response = client
            .get(format!("/job/{}", job.shortcode.as_str()))
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
        let response = client
            .get(format!("/job/raw/{}", job.shortcode.as_str()))
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);

        // Get job when the password is provided
        let response = client
            .post(format!("/job/{}", job.shortcode.as_str()))
            .header(EscrowIdType::Form)
            .body("password=123")
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        // Get job when the password is provided
        let response = client
            .get(format!("/job/raw/{}", job.shortcode.as_str()))
            .cookie(Cookie::new("password", "123"))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        // Get job when the password is provided, but incorrect
        let response = client
            .get(format!("/job/raw/{}", job.shortcode.as_str()))
            .cookie(Cookie::new("password", "abc"))
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
    }
}
