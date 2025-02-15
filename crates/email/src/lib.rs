use std::sync::Arc;

use handlebars::Handlebars;
use resend_rs::types::CreateEmailBaseOptions;
use resend_rs::Resend;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::State;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use serde_json::Value;

#[derive(Clone)]
pub struct ResendMailer {
    pub client: Arc<Resend>,
    pub from_email: String,
    pub templates: Arc<Handlebars<'static>>,
}

impl ResendMailer {
    pub fn new(api_key: String, from_email: String) -> Self {
        Self {
            client: Arc::new(Resend::new(&api_key)),
            from_email,
            templates: Arc::new(Handlebars::new()),
        }
    }

    pub fn load_templates(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Arc::get_mut(&mut self.templates)
            .ok_or_else(|| Box::<dyn std::error::Error>::from("Failed to get mutable reference to templates"))?
            .register_templates_directory("assets_email/templates", Default::default())?;
        Ok(())
    }

    pub async fn send_email(
        &self,
        to_email: Vec<&str>,
        subject: &str,
        template_name: &str,
        data: Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let html = self.templates.render(template_name, &data)?;
        let options = CreateEmailBaseOptions::new(&self.from_email, to_email, subject).with_html(&html);

        self.client.emails.send(options).await?;
        Ok(())
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ResendMailer {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        match request.guard::<&State<ResendMailer>>().await {
            Outcome::Success(state) => Outcome::Success(state.inner().clone()),
            Outcome::Error(_) | Outcome::Forward(_) => Outcome::Error((Status::InternalServerError, ())),
        }
    }
}

impl OpenApiFromRequest<'_> for ResendMailer {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

