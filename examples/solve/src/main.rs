use std::path::PathBuf;

use structopt::StructOpt;

const API_REQUEST_URL: &'static str = "http://2captcha.com/in.php";
const API_RESULT_URL: &'static str = "http://2captcha.com/res.php";

#[derive(Debug, StructOpt)]
struct CliArgs {
    /// 2captcha api key
    #[structopt(short = "a", long = "api-key")]
    api_key: String,
    /// captcha image file
    #[structopt(short = "f", long = "captcha-file")]
    captcha_file: PathBuf,
    /// is captcha is case sensitive
    #[structopt(short = "s", long = "case-sensitive")]
    case_sensitive: bool,
    /// api request url
    #[structopt(long = "api-request-url", default_value = API_REQUEST_URL)]
    api_request_url: String,
    /// api result url
    #[structopt(long = "api-result-url", default_value = API_RESULT_URL)]
    api_result_url: String,
    /// results poll interval timeout (in milliseconds)
    #[structopt(long = "poll-timeout-ms", default_value = "5000")]
    poll_timeout_ms: u64,
}

#[derive(Debug)]
enum Error {
    TwoCaptchaNormal(two_captcha::normal::BuilderError),
    TwoCaptcha(two_captcha::ApiError<two_captcha::normal::PrepareRequestError>),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init_timed();
    let cli_args = CliArgs::from_args();
    log::debug!("cli_args = {:?}", cli_args);

    let captcha = two_captcha::normal::CaptchaBuilder::new()
        .set_upload_file(cli_args.captcha_file)
        .set_case_sensitive(cli_args.case_sensitive)
        .finish()
        .map_err(Error::TwoCaptchaNormal)?;

    let api = two_captcha::Api::new(
        cli_args.api_key.into(),
        two_captcha::Params {
            api_request_url: cli_args.api_request_url,
            api_result_url: cli_args.api_result_url,
            poll_timeout_ms: cli_args.poll_timeout_ms,
        },
    );

    let solved = api.solve(&captcha).await
        .map_err(Error::TwoCaptcha)?;

    println!("{}", solved.answer());

    Ok(())
}
