use crate::{config::Config, Result};
use cached::{Cached, TimedSizedCache};
use lettre::{
    message::{header::ContentType, Message},
    transport::smtp::{authentication::Credentials, client::Tls},
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use nanoid::nanoid;
use std::sync::Mutex;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingTeamVerification {
    pub name: String,
    pub email: String,
}

pub struct EmailService {
    mailer: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from_email: String,
    app_base_url: String,
    verification_tokens: Mutex<TimedSizedCache<String, PendingTeamVerification>>,
}

impl EmailService {
    pub fn new(config: &Config) -> Self {
        let mailer = if config.smtp_url.is_empty() {
            None
        } else {
            match Self::create_mailer(&config.smtp_url) {
                Ok(mailer) => Some(mailer),
                Err(e) => {
                    log::error!("Failed to create mailer: {}", e);
                    None
                }
            }
        };

        Self {
            mailer,
            from_email: config.from_email.clone(),
            app_base_url: config.cors_origin.clone(), // :nauseated_face:
            verification_tokens: Mutex::new(TimedSizedCache::with_size_and_lifespan(1000, 600)),
        }
    }

    fn create_mailer(smtp_url: &str) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let url = url::Url::parse(smtp_url).map_err(|_| Self::validation_error())?;

        let host = url.host_str().unwrap_or("localhost");
        let port = url.port().unwrap_or(587);

        let tls_params = lettre::transport::smtp::client::TlsParameters::new(host.to_string())
            .map_err(|_| Self::validation_error())?;

        let mut mailer = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
            .port(port)
            .tls(Tls::Required(tls_params));

        if !url.username().is_empty() {
            if let Some(password) = url.password() {
                mailer = mailer.credentials(Credentials::new(
                    url.username().to_string(),
                    password.to_string(),
                ));
            }
        }

        Ok(mailer.build())
    }

    pub async fn send_verification_email(
        &self,
        to_email_addr: &str,
        team_name_display: &str,
        pending_team_data: PendingTeamVerification,
    ) -> Result<()> {
        let verification_token = nanoid!();

        {
            let mut tokens_cache = self.verification_tokens.lock().unwrap();
            tokens_cache.cache_set(verification_token.clone(), pending_team_data);
        }

        let verification_link =
            format!("{}/verify?token={}", self.app_base_url, verification_token);

        let subject = format!("Verify your email for smileyCTF - {}", team_name_display);
        let body = format!(
            "Hello {},\n\nPlease verify your email address to complete your registration for smileyCTF by clicking the link below:\n{}\n\nThis link will expire in approximately 10 minutes.\n\nIf you did not request this, please ignore this email.",
            team_name_display,
            verification_link
        );

        self.send_email(to_email_addr, &subject, &body).await
    }

    pub async fn consume_pending_verification(
        &self,
        token: &str,
    ) -> Result<PendingTeamVerification> {
        let mut tokens_cache = self.verification_tokens.lock().unwrap();
        tokens_cache
            .cache_remove(token)
            .ok_or(crate::error::Error::InvalidToken)
    }

    pub fn get_pending_verification_details(&self, token: &str) -> Option<PendingTeamVerification> {
        let mut tokens_cache = self.verification_tokens.lock().unwrap();
        tokens_cache.cache_get(token).cloned()
    }

    async fn send_email(&self, to_email: &str, subject: &str, body: &str) -> Result<()> {
        if let Some(ref mailer) = self.mailer {
            let email = Message::builder()
                .from(
                    self.from_email
                        .parse()
                        .map_err(|_| Self::validation_error())?,
                )
                .to(to_email.parse().map_err(|_| Self::validation_error())?)
                .subject(subject)
                .header(ContentType::TEXT_PLAIN)
                .body(body.to_string())
                .map_err(|_| Self::validation_error())?;

            mailer.send(email).await.map(|_| ()).map_err(|e| {
                log::error!("Failed to send email to {}: {}", to_email, e);
                Self::validation_error()
            })
        } else {
            log::info!(
                "=== EMAIL (No SMTP configured) ===\n\
                To: {}\n\
                Subject: {}\n\
                \n\
                {}\n\
                ===================================",
                to_email,
                subject,
                body
            );
            Ok(())
        }
    }

    fn validation_error() -> crate::error::Error {
        crate::error::Error::Validation(validator::ValidationErrors::new())
    }
}
