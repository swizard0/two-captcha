use std::{
    path::{
        Path,
        PathBuf,
    },
};

use tokio::{
    fs::File,
};

use tokio_util::{
    codec::{
        FramedRead,
        BytesCodec,
    },
};

use reqwest::{
    Body,
    RequestBuilder,
    multipart,
};

use async_trait::{
    async_trait,
};

use crate::{
    ApiToken,
    CaptchaRequest,
};

pub struct Captcha {
    captcha_data: CaptchaData,
    is_case_sensitive: bool,
}

pub struct CaptchaBuilder {
    maybe_captcha_data: Option<CaptchaData>,
    is_case_sensitive: bool,
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
            is_case_sensitive: false,
        }
    }

    pub fn set_upload_file<P>(mut self, path: P) -> Self where P: AsRef<Path> {
        self.maybe_captcha_data = Some(CaptchaData::UploadFile(path.as_ref().to_owned()));
        self
    }

    pub fn set_image_data_base64<T>(mut self, base64_str: T) -> Result<Self, BuilderError> where T: AsRef<str> {
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

    pub fn set_image_data_encode_as_base64<T>(mut self, image_data: T) -> Self where T: AsRef<[u8]> {
        let base64_string = base64::encode(image_data.as_ref());
        self.maybe_captcha_data = Some(CaptchaData::Base64(base64_string));
        self
    }

    pub fn set_case_sensitive(mut self, is_case_sensitive: bool) -> Self {
        self.is_case_sensitive = is_case_sensitive;
        self
    }

    pub fn finish(self) -> Result<Captcha, BuilderError> {
        if let Some(captcha_data) = self.maybe_captcha_data {
            Ok(Captcha {
                captcha_data,
                is_case_sensitive: self.is_case_sensitive,
            })
        } else {
            Err(BuilderError::CaptchaImageIsNotProvided)
        }
    }
}

#[derive(Debug)]
pub enum PrepareRequestError {
    CaptchaImageFileOpen { filename: PathBuf, error: std::io::Error, },
}

#[async_trait]
impl CaptchaRequest for Captcha {
    type PrepareRequestError = PrepareRequestError;

    async fn prepare_request(&self, api_token: &ApiToken, request_builder: RequestBuilder) -> Result<RequestBuilder, Self::PrepareRequestError> {
        match &self.captcha_data {
            CaptchaData::UploadFile(path_buf) => {
                let file = File::open(path_buf).await
                    .map_err(|error| {
                        PrepareRequestError::CaptchaImageFileOpen {
                            filename: path_buf.clone(),
                            error,
                        }
                    })?;
                let stream = FramedRead::new(file, BytesCodec::new());

                let image_file_part = multipart::Part::stream(Body::wrap_stream(stream))
                    .file_name(path_buf.to_string_lossy().to_string());

                log::debug!("building UploadFile request with {:?}", path_buf);

                let form = multipart::Form::new()
                    .text("method", "post")
                    .text("key", api_token.key.clone())
                    .text("json", "1")
                    .text("regsense", if self.is_case_sensitive { "1" } else { "0" })
                    .part("file", image_file_part);

                Ok(request_builder.multipart(form))
            },
            CaptchaData::Base64(base64_string) => {

                log::debug!("building Base64 request with captcha base64 = {:?}", base64_string);

                let request_builder = request_builder
                    .form(&[
                        ("method", "base64"),
                        ("key", &*api_token.key),
                        ("json", "1"),
                        ("regsense", if self.is_case_sensitive { "1" } else { "0" }),
                        ("body", &*base64_string),
                    ]);
                Ok(request_builder)
            },
        }
    }
}
