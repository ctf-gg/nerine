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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingEmailUpdate {
    pub team_id: String,
    pub new_email: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum PendingVerification {
    Team(PendingTeamVerification),
    EmailUpdate(PendingEmailUpdate),
}

pub struct EmailService {
    mailer: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from_email: String,
    app_base_url: String,
    verification_tokens: Mutex<TimedSizedCache<String, PendingVerification>>,
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
            tokens_cache.cache_set(
                verification_token.clone(),
                PendingVerification::Team(pending_team_data),
            );
        }

        let verification_link =
            format!("{}/verify?token={}", self.app_base_url, verification_token);

        let subject = "Verify your email for smileyCTF";
        let body = format!(
            "Hello {},\n\nPlease click the link below to finish registering for smileyCTF:\n{}\n\nThis link will expire in approximately 10 minutes.\n\nIf you did not request this, please ignore this email.",
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
        match tokens_cache.cache_remove(token) {
            Some(PendingVerification::Team(data)) => Ok(data),
            Some(_) => Err(crate::error::Error::InvalidToken),
            None => Err(crate::error::Error::InvalidToken),
        }
    }

    pub fn get_pending_verification_details(&self, token: &str) -> Option<PendingVerification> {
        let mut tokens_cache = self.verification_tokens.lock().unwrap();
        tokens_cache.cache_get(token).cloned()
    }

    pub async fn send_email_change_verification_email(
        &self,
        team_id: &str,
        _new_name: &str,
        to_new_email_addr: &str,
    ) -> Result<()> {
        let verification_token = nanoid!();

        let pending_email_update_data = PendingEmailUpdate {
            team_id: team_id.to_string(),
            new_email: to_new_email_addr.to_string(),
        };

        {
            let mut tokens_cache = self.verification_tokens.lock().unwrap();
            tokens_cache.cache_set(
                verification_token.clone(),
                PendingVerification::EmailUpdate(pending_email_update_data),
            );
        }

        let verification_link = format!(
            "{}/verify?token={}",
            self.app_base_url, verification_token
        );

        let subject = "Verify your new email for smileyCTF";
        let body = format!(
            "Hello {},\n\nPlease click the link below to verify your new email address for smileyCTF:\n{}\n\nThis link will expire in approximately 10 minutes.\n\nIf you did not request this, please ignore this email.",
            _new_name, 
            verification_link
        );

        self.send_email(to_new_email_addr, &subject, &body).await
    }

    pub async fn consume_pending_email_update(&self, token: &str) -> Result<PendingEmailUpdate> {
        let mut tokens_cache = self.verification_tokens.lock().unwrap();
        match tokens_cache.cache_remove(token) {
            Some(PendingVerification::EmailUpdate(data)) => Ok(data),
            Some(_) => Err(crate::error::Error::InvalidToken),
            None => Err(crate::error::Error::InvalidToken),
        }
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
