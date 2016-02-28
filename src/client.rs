use std::io::Read;

use url::Url;
use hyper::Client;
use hyper::method::Method;
use hyper::header::{UserAgent, Accept, ContentLength, ContentType, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use rustc_serialize::{Encodable, Decodable};
use rustc_serialize::json::{self, Json, ToJson};

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
            let req = req.header(ContentType(Mime(TopLevel::Application, SubLevel::Json,
                                             vec![(Attr::Charset, Value::Utf8)])))
                         .body(&body);
            try!(req.send())
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

    fn post<D>(&self, endpoint: &str, params: Vec<(&str, &str)>, data: &Json) -> Result<D>
        where D: Decodable,
    {
        self.request(Method::Post, endpoint, params, data)
    }

    /// [情感分析接口](http://docs.bosonnlp.com/sentiment.html)
    ///
    /// ``contents``: 需要做情感分析的文本序列
    ///
    /// ``model``: 使用不同的语料训练的模型
    ///
    /// # Example
    ///
    /// ```
    /// extern crate bosonnlp;
    /// use bosonnlp::BosonNLP;
    ///
    /// fn main() {
    ///     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
    ///     let rs = nlp.sentiment(&vec!["这家味道还不错".to_owned()], "food").unwrap();
    ///     assert_eq!(1, rs.len());
    /// }
    /// ```
    pub fn sentiment(&self, contents: &[String], model: &str) -> Result<Vec<(f32, f32)>> {
        let endpoint = format!("/sentiment/analysis?{}", model);
        let data = contents.to_json();
        self.post::<Vec<(f32, f32)>>(&endpoint, vec![], &data)
    }

    /// [新闻分类接口](http://docs.bosonnlp.com/classify.html)
    ///
    /// ``contents``: 需要做分类的新闻文本序列
    ///
    /// # Example
    ///
    /// ```
    /// extern crate bosonnlp;
    /// use bosonnlp::BosonNLP;
    ///
    /// fn main() {
    ///     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
    ///     let rs = nlp.classify(&vec!["俄否决安理会谴责叙军战机空袭阿勒颇平民".to_owned()]).unwrap();
    ///     assert_eq!(vec![5usize], rs);
    /// }
    /// ```
    pub fn classify(&self, contents: &[String]) -> Result<Vec<usize>> {
        let data = contents.to_json();
        self.post::<Vec<usize>>("/classify/analysis", vec![], &data)
    }

    /// [语义联想接口](http://docs.bosonnlp.com/suggest.html)
    pub fn suggest(&self, word: &str, top_k: usize) -> () {
        unimplemented!();
    }

    /// [关键词提取接口](http://docs.bosonnlp.com/keywords.html)
    pub fn extract_keywords(&self, text: &str, top_k: usize, segmented: bool) -> () {
        unimplemented!();
    }

    /// [依存文法分析接口](http://docs.bosonnlp.com/depparser.html)
    pub fn depparser(&self, contents: &[String]) -> () {
        unimplemented!();
    }

    /// [命名实体识别接口](http://docs.bosonnlp.com/ner.html)
    pub fn ner(&self, contents: &[String], sensitivity: usize, segmented: bool) -> () {
        unimplemented!();
    }

    /// [分词与词性标注接口](http://docs.bosonnlp.com/tag.html)
    pub fn tag(&self, contents: &[String], space_mode: usize, oov_level: usize, t2s: bool, special_char_conv: bool) -> () {
        unimplemented!();
    }

    /// [新闻摘要接口](http://docs.bosonnlp.com/summary.html)
    pub fn summary(&self, title: &str, content: &str, word_limit: f32, not_exceed: bool) -> () {
        unimplemented!();
    }

    /// [文本聚类接口](http://docs.bosonnlp.com/cluster.html)
    pub fn cluster(&self, contents: &[String], task_id: &str, alpha: f32, beta: f32, timeout: usize) -> () {
        unimplemented!();
    }

    /// [典型意见接口](http://docs.bosonnlp.com/comments.html)
    pub fn comments(&self, contents: &[String], task_id: &str, alpha: f32, beta: f32, timeout: usize) -> () {
        unimplemented!();
    }
}
