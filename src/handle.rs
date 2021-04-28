use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use std::io::{BufRead, Write};
use itertools::Itertools;

pub use crate::parser::AtCoderParser;

pub struct AtCoder {
    client: reqwest::Client,
    //for request
    cookie_headers: HeaderMap,
    //from response
    html: Option<String>,
}

impl AtCoder {
    pub fn new() -> AtCoder {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        AtCoder {
            client: client,
            cookie_headers: HeaderMap::new(),
            html: None,
        }
    }

    pub async fn download(&mut self, url: &str, log: bool) -> Result<(), failure::Error> {
        let url = url::Url::parse(url)?;
        if let Ok(cookie_headers) = AtCoder::local_cookie_headers() {
            self.cookie_headers = cookie_headers;
        }
        let resp = self.call_get_request(url.as_str()).await?;
        self.parse_response(resp).await?;

        let parser = AtCoderParser::new(&self.html.as_ref().unwrap());
        let sample_test_cases = parser.sample_cases();

        if let Some(samples) = sample_test_cases {
            AtCoder::create_sample_test_files(&samples)?;
            println!("File Created !");
            if log {
                println!("====== Download Result ======");
                for (idx, (input, output)) in samples.iter().enumerate() {
                    println!("=== Sample Test Case {} ===", idx + 1);
                    println!("Input:\n{}\nOutput:\n{}", input, output);
                }
                println!("=============================");
            }
        } else {
            println!("There is No Sample !");
        }
        Ok(())
    }

    pub async fn login(&mut self, url: &str) -> Result<(), failure::Error> {
        let url = url::Url::parse(url)?;
        let resp = self.call_get_request(url.as_str()).await?;
        self.parse_response(resp).await?;
        let parser = AtCoderParser::new(self.html.as_ref().unwrap());
        //necessary information and parameters to login AtCoder
        let csrf_token = parser.csrf_token().unwrap();
        let (username, password) = AtCoder::username_and_password();
        let params = {
            let mut params = std::collections::HashMap::new();
            params.insert("username", username);
            params.insert("password", password);
            params.insert("csrf_token", csrf_token);
            params
        };
        //make a post request and try to login
        let resp = self.call_post_request(url.as_str(), &params).await?;
        //save your cookie in your local
        AtCoder::save_cookie_in_local(&resp)?;
        Ok(())
    }

    async fn call_post_request(
        &self,
        url: &str,
        params: &std::collections::HashMap<&str, String>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let resp = self
            .client
            .post(url)
            .headers(self.cookie_headers.clone())
            .form(params)
            .send()
            .await?;
        Ok(resp)
    }
    async fn call_get_request(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let resp = self
            .client
            .get(url)
            .headers(self.cookie_headers.clone())
            .send()
            .await?;
        Ok(resp)
    }

    async fn parse_response(&mut self, response: reqwest::Response) -> Result<(), failure::Error> {
        //cookie
        let mut cookie_headers = HeaderMap::new();
        response.cookies().for_each(|cookie| {
            cookie_headers.insert(
                COOKIE,
                HeaderValue::from_str(&format!("{}={}", cookie.name(), cookie.value())).unwrap(),
            );
        });
        self.cookie_headers = cookie_headers;
        self.html = Some(response.text().await?);
        Ok(())
    }

    //utils
    fn create_sample_test_files(test_cases: &[(String, String)]) -> Result<(), failure::Error> {
        let cases: &str = &test_cases.iter().enumerate().map(|(idx, (input, output))| {
            format!(r##"
#[test]
fn sample{}() {{
let testdir = TestDir::new(BIN, "");
let output = testdir
    .cmd()
    .output_with_stdin(r#"{}"#)
    .tee_output()
    .expect_success();
assert_eq!(output.stdout_str(), r#"{}"#);
assert!(output.stderr_str().is_empty());
}}
"##, idx + 1, if input == "\n" {""} else {input}, output)
        }).join("\n");
        let prefix = r#"use cli_test_dir::*;

const BIN: &'static str = "./main";
"#.to_string();
        let mut sample_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("sample_inputs.rs")?;
        sample_file.write_all((prefix + cases).as_bytes())?;
        Ok(())
    }
    fn save_cookie_in_local(response: &reqwest::Response) -> Result<(), failure::Error> {
        let cookies_str = response
            .cookies()
            .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
            .join(";");
        let path = dirs::home_dir().unwrap().join(".atcoder-sample-downloader");
        //create $HOME/.atcoder-sample-downloader
        std::fs::create_dir_all(path.clone())?;
        //create cookie.jar under this directory
        let cookie_path = path.join("cookie.jar");
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(cookie_path.clone())?
            .write_all(cookies_str.as_bytes())?;
        println!("SAVED YOUR COOKIE IN {}", cookie_path.to_str().unwrap());
        Ok(())
    }
    fn username_and_password() -> (String, String) {
        println!("Please input Your username and password");
        let username = rpassword::read_password_from_tty(Some("Username > ")).unwrap();
        let password = rpassword::read_password_from_tty(Some("Password > ")).unwrap();
        (username, password)
    }
    fn local_cookie_headers() -> Result<HeaderMap, failure::Error> {
        let cookiejar_path = dirs::home_dir()
            .unwrap()
            .join(".atcoder-sample-downloader")
            .join("cookie.jar");
        let file = std::fs::File::open(cookiejar_path)?;
        let reader = std::io::BufReader::new(file);

        let mut cookie_headers = HeaderMap::new();
        reader.lines().for_each(|line| {
            cookie_headers.insert(
                COOKIE,
                HeaderValue::from_str(&format!("{}", line.unwrap())).unwrap(),
            );
        });
        Ok(cookie_headers)
    }
}
