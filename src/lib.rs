use serde_derive::{
    Deserialize,
};

use reqwest::{
    Client,
    StatusCode,
    RequestBuilder,
};

use async_trait::{
    async_trait,
};

pub mod normal;

#[derive(Clone, PartialEq, Debug)]
pub struct Params {
    pub api_request_url: String,
    pub api_result_url: String,
    pub poll_timeout_ms: u64,
}

impl Default for Params {
    fn default() -> Params {
        Params {
            api_request_url: "http://2captcha.com/in.php".into(),
            api_result_url: "http://2captcha.com/res.php".into(),
            poll_timeout_ms: 5000,
        }
    }
}

pub struct ApiToken {
    key: String,
}

impl From<String> for ApiToken {
    fn from(key: String) -> ApiToken {
        ApiToken { key, }
    }
}

pub struct Api {
    api_token: ApiToken,
    params: Params,
}

pub struct Solved {
}

#[derive(Debug)]
pub enum ApiError<E> {
    PrepareRequest(E),
    SendRequest(reqwest::Error),
    BadStatusCode { status_code: StatusCode, },
    ReadResponse(reqwest::Error),
    DecodeResponse(serde_json::Error),
    UnexpectedApiRequestResponse(ApiRequestResponse),
    WrongUserKey,
    KeyDoesNotExist,
    ZeroBalance,
    Pageurl,
    NoSlotAvailable,
    ZeroCaptchaFilesize,
    TooBigCaptchaFilesize,
    WrongFileExtension,
    ImageTypeNotSupported,
    Upload,
    IpNotAllowed,
    IpBanned,
    BadTokenOrPageurl,
    Googlekey,
    WrongGooglekey,
    CaptchaimageBlocked,
    TooManyBadImages,
    MaxUserTurn,
    BadParameters,
    BadProxy,
}

impl Api {
    pub fn new(api_token: ApiToken, params: Params) -> Api {
        Api { api_token, params, }
    }

    pub async fn solve<C>(&self, captcha: C) -> Result<Solved, ApiError<C::PrepareRequestError>> where C: CaptchaRequest {
        let client = Client::new();
        let request_builder = client.post(&self.params.api_request_url);
        let request_builder = captcha.prepare_request(&self.api_token, request_builder).await
            .map_err(ApiError::PrepareRequest)?;
        let response = request_builder.send().await
            .map_err(ApiError::SendRequest)?;
        let status_code = response.status();
        if status_code != StatusCode::OK {
            return Err(ApiError::BadStatusCode { status_code, });
        }
        let api_response_string = response.text().await
            .map_err(ApiError::ReadResponse)?;
        let api_response: ApiRequestResponse = serde_json::from_str(&api_response_string)
            .map_err(ApiError::DecodeResponse)?;

        let captcha_id = match api_response {
            ApiRequestResponse { status: 1, request, } =>
                request,
            ApiRequestResponse { status: 0, request, } if request == "ERROR_WRONG_USER_KEY" =>
                return Err(ApiError::WrongUserKey),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_KEY_DOES_NOT_EXIST" =>
                return Err(ApiError::KeyDoesNotExist),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_ZERO_BALANCE" =>
                return Err(ApiError::ZeroBalance),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_PAGEURL" =>
                return Err(ApiError::Pageurl),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_NO_SLOT_AVAILABLE" =>
                return Err(ApiError::NoSlotAvailable),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_ZERO_CAPTCHA_FILESIZE" =>
                return Err(ApiError::ZeroCaptchaFilesize),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_TOO_BIG_CAPTCHA_FILESIZE" =>
                return Err(ApiError::TooBigCaptchaFilesize),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_WRONG_FILE_EXTENSION" =>
                return Err(ApiError::WrongFileExtension),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_IMAGE_TYPE_NOT_SUPPORTED" =>
                return Err(ApiError::ImageTypeNotSupported),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_UPLOAD" =>
                return Err(ApiError::Upload),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_IP_NOT_ALLOWED" =>
                return Err(ApiError::IpNotAllowed),
            ApiRequestResponse { status: 0, request, } if request == "IP_BANNED" =>
                return Err(ApiError::IpBanned),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_BAD_TOKEN_OR_PAGEURL" =>
                return Err(ApiError::BadTokenOrPageurl),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_GOOGLEKEY" =>
                return Err(ApiError::Googlekey),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_WRONG_GOOGLEKEY" =>
                return Err(ApiError::WrongGooglekey),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_CAPTCHAIMAGE_BLOCKED" =>
                return Err(ApiError::CaptchaimageBlocked),
            ApiRequestResponse { status: 0, request, } if request == "TOO_MANY_BAD_IMAGES" =>
                return Err(ApiError::TooManyBadImages),
            ApiRequestResponse { status: 0, request, } if request == "MAX_USER_TURN" =>
                return Err(ApiError::MaxUserTurn),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_BAD_PARAMETERS" =>
                return Err(ApiError::BadParameters),
            ApiRequestResponse { status: 0, request, } if request == "ERROR_BAD_PROXY" =>
                return Err(ApiError::BadProxy),
            other =>
                return Err(ApiError::UnexpectedApiRequestResponse(other)),
        };

        todo!()
    }
}

#[async_trait]
pub trait CaptchaRequest {
    type PrepareRequestError;

    async fn prepare_request(&self, api_token: &ApiToken, request_builder: RequestBuilder) -> Result<RequestBuilder, Self::PrepareRequestError>;
}

#[derive(Deserialize, Debug)]
pub struct ApiRequestResponse {
    pub status: i32,
    pub request: String,
}
