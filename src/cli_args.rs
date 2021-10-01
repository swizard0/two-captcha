use structopt::{
    clap::{
        AppSettings,
    },
    StructOpt,
};

#[derive(Clone, StructOpt, Debug)]
#[structopt(setting = AppSettings::DeriveDisplayOrder)]
pub struct CliArgs {
    /// 2captcha api request url
    #[structopt(long = "two-captcha-api-request-url", default_value = crate::API_REQUEST_URL)]
    api_request_url: String,
    /// 2captcha api result url
    #[structopt(long = "two-captcha-api-result-url", default_value = crate::API_RESULT_URL)]
    api_result_url: String,
    /// 2captcha results poll interval timeout (in milliseconds)
    #[structopt(long = "two-captcha-poll-timeout-ms", default_value = crate::DEFAULT_POLL_TIMEOUT_MS_STR)]
    poll_timeout_ms: u64,
}

impl crate::Params {
    pub fn from_cli_args<A>(cli_args: A) -> Self where A: AsRef<CliArgs> {
        Self {
            api_request_url: cli_args.as_ref().api_request_url.clone(),
            api_result_url: cli_args.as_ref().api_request_url.clone(),
            poll_timeout_ms: cli_args.as_ref().poll_timeout_ms,
        }
    }
}
