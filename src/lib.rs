use reqwest::{
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
pub enum ApiError {
}

impl Api {
    pub fn new(api_token: ApiToken, params: Params) -> Api {
        Api { api_token, params, }
    }

    pub fn solve<C>(&self, captcha: C) -> Result<Solved, ApiError> where C: CaptchaRequest {

        todo!()
    }
}

#[async_trait]
pub trait CaptchaRequest {
    type PrepareRequestError;

    async fn prepare_request(&self, api_token: &ApiToken, request_builder: RequestBuilder) -> Result<RequestBuilder, Self::PrepareRequestError>;
}
