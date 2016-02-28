use std::io::Read;

use url::Url;
use hyper::Client;
use hyper::method::Method;
use hyper::header::{UserAgent, Accept, ContentLength, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use rustc_serialize::{Encodable, Decodable};
use rustc_serialize::json::{self, Json};

use errors::Error;


const DEFAULT_BOSONNLP_URL: &'static str = "http://api.bosonnlp.com";

header! { (XToken, "X-Token") => [String] }

pub type Result<T> = ::std::result::Result<T, Error>;

/// BosonNLP REST API 访问的封装
pub struct BosonNLP {
    /// 用于 API 鉴权的 API Token
    pub token: String,
    /// 是否压缩大于 10K 的请求体，默认为 true
    pub compress: bool,
    /// BosonNLP HTTP API 的 URL，默认为 `http://api.bosonnlp.com`
    bosonnlp_url: String,
    /// hyper http Client
    client: Client,
}

impl BosonNLP {
    /// 初始化一个新的 ``BosonNLP`` 实例
    pub fn new<T: Into<String>>(token: T) -> BosonNLP {
        BosonNLP {
            token: token.into(),
            compress: true,
            bosonnlp_url: DEFAULT_BOSONNLP_URL.to_owned(),
            client: Client::new(),
        }
    }

    /// 使用自定义参数初始化一个新的 ``BosonNLP`` 实例
    pub fn with_options<T: Into<String>>(token: T, bosonnlp_url: T, compress: bool) -> BosonNLP {
        BosonNLP {
            token: token.into(),
            compress: compress,
            bosonnlp_url: bosonnlp_url.into(),
            client: Client::new(),
        }
    }

    /// 使用自定义的 hyper Client 初始化一个新的 ``BosonNLP`` 实例
    pub fn with_client<T: Into<String>>(token: T, client: Client) -> BosonNLP {
        BosonNLP {
            token: token.into(),
            compress: true,
            bosonnlp_url: DEFAULT_BOSONNLP_URL.to_owned(),
            client: client,
        }
    }

    fn request<D, E>(&self, method: Method, endpoint: &str, params: Vec<(&str, &str)>, data: &E) -> Result<D>
        where D: Decodable,
              E: Encodable
    {
        let url_string = format!("{}{}", self.bosonnlp_url, endpoint);
        let mut url = Url::parse(&url_string).unwrap();
        url.set_query_from_pairs(params.into_iter());
        let body = match json::encode(data) {
            Ok(d) => d,
            Err(..) => "".to_owned(),
        };
        let req = self.client.request(method.clone(), url)
                                 .header(UserAgent(format!("bosonnlp-rs/{}", env!("CARGO_PKG_VERSION"))))
                                 .header(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Json,
                                                                vec![(Attr::Charset, Value::Utf8)]))]))
                                 .header(XToken(self.token.clone()));
        let mut res = if method == Method::Post {
            try!(req.body(&body).send())
        } else {
            try!(req.send())
        };
        let mut body = match res.headers.clone().get::<ContentLength>() {
            Some(&ContentLength(len)) => String::with_capacity(len as usize),
            _ => String::new(),
        };
        try!(res.read_to_string(&mut body));
        debug!("rev response {:#?} {:#?}", res.status, body);
        if !res.status.is_success() {
            let result = match Json::from_str(&body) {
                Ok(obj) => obj,
                Err(..) => Json::Object(json::Object::new()),
            };
            let message = match result.find("message") {
                Some(msg) => msg.as_string().unwrap_or("").to_owned(),
                None => "".to_owned(),
            };
            return Err(Error::Api { code: res.status, reason: message });
        }
        Ok(try!(json::decode::<D>(&body)))
    }

    fn get<D: Decodable>(&self, endpoint: &str, params: Vec<(&str, &str)>) -> Result<D> {
        self.request(Method::Get, endpoint, params, &json::Object::new())
    }

    fn post<D, E>(&self, endpoint: &str, params: Vec<(&str, &str)>, data: &E) -> Result<D>
        where D: Decodable,
              E: Encodable
    {
        self.request(Method::Post, endpoint, params, data)
    }
}
