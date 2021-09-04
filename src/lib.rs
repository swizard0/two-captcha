
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

pub struct Api {
    api_token: String,
    params: Params,
}

pub struct Solved {
}

#[derive(Debug)]
pub enum ApiError {

}

impl Api {
    pub fn new(api_token: String, params: Params) -> Api {
        Api { api_token, params, }
    }

    pub fn solve<C>(&self, captcha: C) -> Result<Solved, ApiError> {

        todo!()
    }
}
