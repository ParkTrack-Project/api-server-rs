use std::{future::Future, pin::Pin};

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox,
    transport::smtp::authentication::Credentials,
};

use crate::{
    error::{ApiError, ApiResult},
    state::ApiConfig,
};

pub trait EmailService: Send + Sync {
    fn send_password_reset_email<'a>(
        &'a self,
        email: &'a str,
        token: &'a str,
        config: &'a ApiConfig,
    ) -> Pin<Box<dyn Future<Output = ApiResult<()>> + Send + 'a>>;
}

pub struct SmtpEmailService;

impl EmailService for SmtpEmailService {
    fn send_password_reset_email<'a>(
        &'a self,
        email: &'a str,
        token: &'a str,
        config: &'a ApiConfig,
    ) -> Pin<Box<dyn Future<Output = ApiResult<()>> + Send + 'a>> {
        Box::pin(async move {
            let message = Message::builder()
                .from(Mailbox::new(
                    Some(config.smtp_from_name.clone()),
                    config
                        .smtp_from_email
                        .parse()
                        .map_err(anyhow::Error::from)?,
                ))
                .to(email.parse().map_err(anyhow::Error::from)?)
                .subject("Password reset")
                .body(format!(
                    "Your password reset token: {token}\nValid for {} minutes.\n{}",
                    config.password_reset_ttl_minutes, config.password_reset_login_url
                ))
                .map_err(anyhow::Error::from)?;

            let builder = if config.smtp_use_ssl {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
                    .map_err(anyhow::Error::from)?
            } else if config.smtp_use_tls {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
                    .map_err(anyhow::Error::from)?
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
            };
            let mut builder =
                builder.port(u16::try_from(config.smtp_port).map_err(anyhow::Error::from)?);
            if !config.smtp_username.is_empty() {
                builder = builder.credentials(Credentials::new(
                    config.smtp_username.clone(),
                    config.smtp_password.clone(),
                ));
            }
            builder
                .build()
                .send(message)
                .await
                .map_err(|_| ApiError::ServiceUnavailable("email delivery failed".into()))?;
            Ok(())
        })
    }
}
