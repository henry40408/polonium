use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub(crate) enum AttachmentError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("URL error: {0}")]
    UrlError(#[from] url::ParseError),
    #[error("unknown MIME type")]
    MIMEInferError,
}

pub(crate) struct Attachment {
    filename: String,
    mime_type: String,
    content: Vec<u8>,
}

impl Attachment {
    fn new(filename: &str, mime_type: &str, content: &[u8]) -> Self {
        Self {
            filename: filename.into(),
            mime_type: mime_type.into(),
            content: content.into(),
        }
    }

    async fn from_url(url: &str) -> Result<Self, AttachmentError> {
        let parsed = Url::parse(url)?;
        let filename = parsed
            .path_segments()
            .map_or("filename", |t| t.last().map_or("filename", |t1| t1));

        let res = reqwest::get(url).await?;
        let buffer = res.bytes().await?.to_vec();

        let mime_type = infer::get(&buffer).ok_or(AttachmentError::MIMEInferError)?;

        Ok(Self {
            filename: filename.to_string(),
            mime_type: mime_type.to_string(),
            content: buffer,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Notification;
    use crate::{Attachment, AttachmentError};

    #[test]
    fn test_attachment_new() {
        Attachment::new("filename", "plain/text", &[]);
    }

    #[test]
    fn test_attach() {
        let mut n = build_notification();
        let a = Attachment::new("filename", "plain/text", &[]);
        n.attach(&a);
    }

    #[tokio::test]
    async fn test_attach_url() -> Result<(), AttachmentError> {
        let u = "https://upload.wikimedia.org/wikipedia/commons/1/1a/1x1_placeholder.png";
        let a = Attachment::from_url(u).await?;
        assert_eq!("1x1_placeholder.png", a.filename);
        assert_eq!("image/png", a.mime_type);
        assert!(a.content.len() > 0);
        Ok(())
    }

    fn build_notification<'a>() -> Notification<'a> {
        let user = "user";
        let token = "token";
        let message = "message";
        Notification::new(token, user, message)
    }
}
