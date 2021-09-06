use std::{
    time::{
        Instant,
        Duration,
    },
};

use serde_derive::{
    Deserialize,
};

use reqwest::{
    Client,
    StatusCode,
    RequestBuilder,
};

use tokio::{
    time::{
        sleep,
    },
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
    answer: String,
}

impl Solved {
    pub fn answer(&self) -> &str {
        &self.answer
    }
}

#[derive(Debug)]
pub enum ApiError<E> {
    PrepareCaptchaRequest(E),
    SendCaptchaRequest(reqwest::Error),
    SendCaptchaRequestBadStatusCode { status_code: StatusCode, },
    ReadCaptchaResponse(reqwest::Error),
    DecodeCaptchaResponse(serde_json::Error),
    CaptchaResponse(CaptchaResponseError),
    PollResponse(PollResponseError),
    SendPollRequest(reqwest::Error),
    SendPollRequestBadStatusCode { status_code: StatusCode, },
    ReadPollResponse(reqwest::Error),
    DecodePollResponse(serde_json::Error),
}

impl Api {
    pub fn new(api_token: ApiToken, params: Params) -> Api {
        Api { api_token, params, }
    }

    pub async fn solve<C>(&self, captcha: C) -> Result<Solved, ApiError<C::PrepareRequestError>> where C: CaptchaRequest {
        let client = Client::new();
        let request_builder = client.post(&self.params.api_request_url);
        let request_builder = captcha.prepare_request(&self.api_token, request_builder).await
            .map_err(ApiError::PrepareCaptchaRequest)?;
        let response = request_builder.send().await
            .map_err(ApiError::SendCaptchaRequest)?;
        let status_code = response.status();
        if status_code != StatusCode::OK {
            return Err(ApiError::SendCaptchaRequestBadStatusCode { status_code, });
        }
        let api_response_string = response.text().await
            .map_err(ApiError::ReadCaptchaResponse)?;
        let api_response: ApiResponse = serde_json::from_str(&api_response_string)
            .map_err(ApiError::DecodeCaptchaResponse)?;

        let captcha_id = api_response.extract_captcha_id()
            .map_err(ApiError::CaptchaResponse)?;

        let get_parameters = [
            ("key", &*self.api_token.key),
            ("action", "get"),
            ("id", &captcha_id),
            ("json", "1"),
        ];

        loop {
            let now = Instant::now();

            let response = client.get(&self.params.api_result_url)
                .form(&get_parameters)
                .send()
                .await
                .map_err(ApiError::SendPollRequest)?;
            let status_code = response.status();
            if status_code != StatusCode::OK {
                return Err(ApiError::SendPollRequestBadStatusCode { status_code, });
            }
            let api_response_string = response.text().await
                .map_err(ApiError::ReadPollResponse)?;
            let api_response: ApiResponse = serde_json::from_str(&api_response_string)
                .map_err(ApiError::DecodePollResponse)?;

            let poll_result = api_response.extract_poll_result()
                .map_err(ApiError::PollResponse)?;
            match poll_result {
                PollResult::NotReady => {
                    let elapsed = now.elapsed().as_millis() as u64;
                    if elapsed < self.params.poll_timeout_ms {
                        sleep(Duration::from_millis(self.params.poll_timeout_ms - elapsed)).await;
                    }
                },
                PollResult::Ready { solved_captcha, } =>
                    return Ok(Solved { answer: solved_captcha, }),
            }
        }
    }
}

#[async_trait]
pub trait CaptchaRequest {
    type PrepareRequestError;

    async fn prepare_request(&self, api_token: &ApiToken, request_builder: RequestBuilder) -> Result<RequestBuilder, Self::PrepareRequestError>;
}

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub status: i32,
    pub request: String,
}

#[derive(Debug)]
pub enum CaptchaResponseError {
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
    UnexpectedApiResponse(ApiResponse),
}

#[derive(Debug)]
pub enum PollResponseError {
    ErrorCaptchaUnsolvable,
    ErrorWrongUserKey,
    ErrorKeyDoesNotExist,
    ErrorWrongIdFormat,
    ErrorWrongCaptchaId,
    ErrorBadDuplicates,
    ErrorReportNotRecorded,
    ErrorDuplicateReport,
    RequestLimitExceeded { code: String, },
    ErrorIpAddres,
    ErrorTokenExpired,
    ErrorEmptyAction,
    ErrorProxyConnectionFailed,
    UnexpectedApiResponse(ApiResponse),
}

enum PollResult {
    NotReady,
    Ready { solved_captcha: String, },
}

impl ApiResponse {
    fn extract_captcha_id(self) -> Result<String, CaptchaResponseError> {
        match self {
            ApiResponse { status: 1, request, } =>
                Ok(request),
            ApiResponse { status: 0, request, } if request == "ERROR_WRONG_USER_KEY" =>
                Err(CaptchaResponseError::WrongUserKey),
            ApiResponse { status: 0, request, } if request == "ERROR_KEY_DOES_NOT_EXIST" =>
                Err(CaptchaResponseError::KeyDoesNotExist),
            ApiResponse { status: 0, request, } if request == "ERROR_ZERO_BALANCE" =>
                Err(CaptchaResponseError::ZeroBalance),
            ApiResponse { status: 0, request, } if request == "ERROR_PAGEURL" =>
                Err(CaptchaResponseError::Pageurl),
            ApiResponse { status: 0, request, } if request == "ERROR_NO_SLOT_AVAILABLE" =>
                Err(CaptchaResponseError::NoSlotAvailable),
            ApiResponse { status: 0, request, } if request == "ERROR_ZERO_CAPTCHA_FILESIZE" =>
                Err(CaptchaResponseError::ZeroCaptchaFilesize),
            ApiResponse { status: 0, request, } if request == "ERROR_TOO_BIG_CAPTCHA_FILESIZE" =>
                Err(CaptchaResponseError::TooBigCaptchaFilesize),
            ApiResponse { status: 0, request, } if request == "ERROR_WRONG_FILE_EXTENSION" =>
                Err(CaptchaResponseError::WrongFileExtension),
            ApiResponse { status: 0, request, } if request == "ERROR_IMAGE_TYPE_NOT_SUPPORTED" =>
                Err(CaptchaResponseError::ImageTypeNotSupported),
            ApiResponse { status: 0, request, } if request == "ERROR_UPLOAD" =>
                Err(CaptchaResponseError::Upload),
            ApiResponse { status: 0, request, } if request == "ERROR_IP_NOT_ALLOWED" =>
                Err(CaptchaResponseError::IpNotAllowed),
            ApiResponse { status: 0, request, } if request == "IP_BANNED" =>
                Err(CaptchaResponseError::IpBanned),
            ApiResponse { status: 0, request, } if request == "ERROR_BAD_TOKEN_OR_PAGEURL" =>
                Err(CaptchaResponseError::BadTokenOrPageurl),
            ApiResponse { status: 0, request, } if request == "ERROR_GOOGLEKEY" =>
                Err(CaptchaResponseError::Googlekey),
            ApiResponse { status: 0, request, } if request == "ERROR_WRONG_GOOGLEKEY" =>
                Err(CaptchaResponseError::WrongGooglekey),
            ApiResponse { status: 0, request, } if request == "ERROR_CAPTCHAIMAGE_BLOCKED" =>
                Err(CaptchaResponseError::CaptchaimageBlocked),
            ApiResponse { status: 0, request, } if request == "TOO_MANY_BAD_IMAGES" =>
                Err(CaptchaResponseError::TooManyBadImages),
            ApiResponse { status: 0, request, } if request == "MAX_USER_TURN" =>
                Err(CaptchaResponseError::MaxUserTurn),
            ApiResponse { status: 0, request, } if request == "ERROR_BAD_PARAMETERS" =>
                Err(CaptchaResponseError::BadParameters),
            ApiResponse { status: 0, request, } if request == "ERROR_BAD_PROXY" =>
                Err(CaptchaResponseError::BadProxy),
            other =>
                Err(CaptchaResponseError::UnexpectedApiResponse(other)),
        }
    }

    fn extract_poll_result(self) -> Result<PollResult, PollResponseError> {
        match self {
            ApiResponse { status: 1, request, } =>
                Ok(PollResult::Ready { solved_captcha: request, }),
            ApiResponse { status: 0, request, } if request == "CAPCHA_NOT_READY" =>
                Ok(PollResult::NotReady),
            ApiResponse { status: 0, request, } if request == "ERROR_CAPTCHA_UNSOLVABLE" =>
                Err(PollResponseError::ErrorCaptchaUnsolvable),
            ApiResponse { status: 0, request, } if request == "ERROR_WRONG_USER_KEY" =>
                Err(PollResponseError::ErrorWrongUserKey),
            ApiResponse { status: 0, request, } if request == "ERROR_KEY_DOES_NOT_EXIST" =>
                Err(PollResponseError::ErrorKeyDoesNotExist),
            ApiResponse { status: 0, request, } if request == "ERROR_WRONG_ID_FORMAT" =>
                Err(PollResponseError::ErrorWrongIdFormat),
            ApiResponse { status: 0, request, } if request == "ERROR_WRONG_CAPTCHA_ID" =>
                Err(PollResponseError::ErrorWrongCaptchaId),
            ApiResponse { status: 0, request, } if request == "ERROR_BAD_DUPLICATES" =>
                Err(PollResponseError::ErrorBadDuplicates),
            ApiResponse { status: 0, request, } if request == "ERROR_REPORT_NOT_RECORDED" =>
                Err(PollResponseError::ErrorReportNotRecorded),
            ApiResponse { status: 0, request, } if request == "ERROR_DUPLICATE_REPORT" =>
                Err(PollResponseError::ErrorDuplicateReport),
            ApiResponse { status: 0, request, } if request.starts_with("ERROR:") =>
                Err(PollResponseError::RequestLimitExceeded { code: request[6 ..].trim().to_string(), }),
            ApiResponse { status: 0, request, } if request == "ERROR_IP_ADDRES" =>
                Err(PollResponseError::ErrorIpAddres),
            ApiResponse { status: 0, request, } if request == "ERROR_TOKEN_EXPIRED" =>
                Err(PollResponseError::ErrorTokenExpired),
            ApiResponse { status: 0, request, } if request == "ERROR_EMPTY_ACTION" =>
                Err(PollResponseError::ErrorEmptyAction),
            ApiResponse { status: 0, request, } if request == "ERROR_PROXY_CONNECTION_FAILED" =>
                Err(PollResponseError::ErrorProxyConnectionFailed),
            other =>
                Err(PollResponseError::UnexpectedApiResponse(other)),
        }
    }
}
