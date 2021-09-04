use std::{
    path::{
        Path,
        PathBuf,
    },
};

pub struct Captcha {
    captcha_data: CaptchaData,
}

pub struct CaptchaBuilder {
    maybe_captcha_data: Option<CaptchaData>,
}

enum CaptchaData {
    UploadFile(PathBuf),
    Base64(String),
}

#[derive(Debug)]
pub enum BuilderError {
    InvalidBase64 { source: String, error: base64::DecodeError, },
    CaptchaImageIsNotProvided,
}

impl CaptchaBuilder {
    pub fn new() -> CaptchaBuilder {
        CaptchaBuilder {
            maybe_captcha_data: None,
        }
    }

    pub fn set_upload_file<P>(&mut self, path: P) -> &mut Self where P: AsRef<Path> {
        self.maybe_captcha_data = Some(CaptchaData::UploadFile(path.as_ref().to_owned()));
        self
    }

    pub fn set_image_data_base64<T>(&mut self, base64_str: T) -> Result<&mut Self, BuilderError> where T: AsRef<str> {
        base64::decode(base64_str.as_ref())
            .map_err(|error| {
                BuilderError::InvalidBase64 {
                    source: base64_str.as_ref().to_string(),
                    error,
                }
            })?;
        self.maybe_captcha_data = Some(CaptchaData::Base64(base64_str.as_ref().to_string()));
        Ok(self)
    }

    pub fn set_image_data_encode_as_base64<T>(&mut self, image_data: T) -> &mut Self where T: AsRef<[u8]> {
        let base64_string = base64::encode(image_data.as_ref());
        self.maybe_captcha_data = Some(CaptchaData::Base64(base64_string));
        self
    }

    pub fn finish(self) -> Result<Captcha, BuilderError> {
        if let Some(captcha_data) = self.maybe_captcha_data {
            Ok(Captcha { captcha_data, })
        } else {
            Err(BuilderError::CaptchaImageIsNotProvided)
        }
    }
}
