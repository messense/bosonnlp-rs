use std::io::{Read, Write};

use jsonway;
use url::Url;
use uuid::Uuid;
use flate2::Compression;
use flate2::write::GzEncoder;
use hyper::Client;
use hyper::method::Method;
use hyper::header::{UserAgent, Accept, ContentLength, ContentType, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use rustc_serialize::{Encodable, Decodable};
use rustc_serialize::json::{self, Json, ToJson};

use errors::Error;
use rep::{Dependency, NamedEntity, Tag, ClusterContent, TextCluster, CommentsCluster};
use task::{ClusterTask, CommentsTask, Task};


/// 默认的 BosonNLP API 服务器地址
const DEFAULT_BOSONNLP_URL: &'static str = "http://api.bosonnlp.com";

/// BosonNLP API 鉴权 HTTP Header
header! { (XToken, "X-Token") => [String] }

/// BosonNLP API 返回结果类型
pub type Result<T> = ::std::result::Result<T, Error>;

/// [BosonNLP](http://bosonnlp.com) REST API 访问的封装
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
        let body;
        let compressed;
        let req = self.client
                      .request(method.clone(), url)
                      .header(UserAgent(format!("bosonnlp-rs/{}", env!("CARGO_PKG_VERSION"))))
                      .header(Accept(vec![qitem(Mime(TopLevel::Application,
                                                     SubLevel::Json,
                                                     vec![(Attr::Charset, Value::Utf8)]))]))
                      .header(XToken(self.token.clone()));
        let mut res = if method == Method::Post {
            let req = req.header(ContentType(Mime(TopLevel::Application,
                                                  SubLevel::Json,
                                                  vec![(Attr::Charset, Value::Utf8)])));
            body = match json::encode(data) {
                Ok(d) => d,
                Err(..) => "".to_owned(),
            };
            if self.compress && body.len() > 10240 {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::Default);
                try!(encoder.write(body.as_bytes()));
                compressed = try!(encoder.finish());
                try!(req.body(&compressed[..]).send())
            } else {
                try!(req.body(&body).send())
            }
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
                None => body,
            };
            return Err(Error::Api {
                code: res.status,
                reason: message,
            });
        }
        Ok(try!(json::decode::<D>(&body)))
    }

    pub fn get<D: Decodable>(&self, endpoint: &str, params: Vec<(&str, &str)>) -> Result<D> {
        self.request(Method::Get, endpoint, params, &json::Object::new())
    }

    pub fn post<D>(&self, endpoint: &str, params: Vec<(&str, &str)>, data: &Json) -> Result<D>
        where D: Decodable
    {
        self.request(Method::Post, endpoint, params, data)
    }

    /// [情感分析接口](http://docs.bosonnlp.com/sentiment.html)
    ///
    /// ``contents``: 需要做情感分析的文本序列
    ///
    /// ``model``: 使用不同的语料训练的模型
    ///
    /// # 使用示例
    ///
    /// ```
    /// extern crate bosonnlp;
    ///
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
    /// # 使用示例
    ///
    /// ```
    /// extern crate bosonnlp;
    ///
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
    ///
    /// ``word``: 需要做语义联想的词
    ///
    /// ``top_k``: 返回结果的条数，最大值可设定为 100
    ///
    /// # 使用示例
    ///
    /// ```
    /// extern crate bosonnlp;
    ///
    /// use bosonnlp::BosonNLP;
    ///
    /// fn main() {
    ///     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
    ///     let rs = nlp.suggest("北京", 2).unwrap();
    ///     assert_eq!(2, rs.len());
    /// }
    /// ```
    pub fn suggest<T: AsRef<str>>(&self, word: T, top_k: usize) -> Result<Vec<(f32, String)>> {
        let data = word.as_ref().to_json();
        self.post::<Vec<(f32, String)>>("/suggest/analysis",
                                        vec![("top_k", &top_k.to_string())],
                                        &data)
    }

    /// [关键词提取接口](http://docs.bosonnlp.com/keywords.html)
    ///
    /// ``text``: 需要做关键词提取的文本
    ///
    /// ``top_k``: 返回结果的条数，最大值可设定为 100
    ///
    /// ``segmented``: `text` 是否已经进行了分词，若为 `true` 则不会再对内容进行分词处理
    ///
    /// # 使用示例
    ///
    /// ```
    /// extern crate bosonnlp;
    ///
    /// use bosonnlp::BosonNLP;
    ///
    /// fn main() {
    ///     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
    ///     let rs = nlp.keywords("病毒式媒体网站：让新闻迅速蔓延", 2, false).unwrap();
    ///     assert_eq!(2, rs.len());
    /// }
    /// ```
    pub fn keywords<T: AsRef<str>>(&self, text: T, top_k: usize, segmented: bool) -> Result<Vec<(f32, String)>> {
        let data = text.as_ref().to_json();
        let top_k_str = top_k.to_string();
        let params = match segmented {
            true => vec![("top_k", top_k_str.as_ref()), ("segmented", "1")],
            false => vec![("top_k", top_k_str.as_ref())],
        };
        self.post::<Vec<(f32, String)>>("/keywords/analysis", params, &data)
    }

    /// [依存文法分析接口](http://docs.bosonnlp.com/depparser.html)
    ///
    /// ``contents``: 需要做依存文法分析的文本序列
    ///
    /// # 使用示例
    ///
    /// ```
    /// extern crate bosonnlp;
    ///
    /// use bosonnlp::BosonNLP;
    ///
    /// fn main() {
    ///     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
    ///     let rs = nlp.depparser(&vec!["今天天气好".to_owned()]).unwrap();
    ///     assert_eq!(1, rs.len());
    ///     let dep0 = &rs[0];
    ///     assert_eq!(vec![2isize, 2isize, -1isize], dep0.head);
    ///     let rs = nlp.depparser(&vec!["今天天气好".to_owned(), "美好的世界".to_owned()]).unwrap();
    ///     assert_eq!(2, rs.len());
    /// }
    /// ```
    pub fn depparser(&self, contents: &[String]) -> Result<Vec<Dependency>> {
        let data = contents.to_json();
        self.post::<Vec<Dependency>>("/depparser/analysis", vec![], &data)
    }

    /// [命名实体识别接口](http://docs.bosonnlp.com/ner.html)
    ///
    /// ``contents``: 需要做命名实体识别的文本序列
    ///
    /// ``sensitivity``: 准确率与召回率之间的平衡。
    /// 设置成 1 能找到更多的实体，设置成 5 能以更高的精度寻找实体
    /// 一般设置为 3
    ///
    /// ``segmented``: 输入是否已经为分词结果
    ///
    /// # 使用示例
    ///
    /// ```
    /// extern crate bosonnlp;
    ///
    /// use bosonnlp::BosonNLP;
    ///
    /// fn main() {
    ///     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
    ///     let rs = nlp.ner(&vec!["成都商报记者 姚永忠".to_owned()], 2, false).unwrap();
    ///     assert_eq!(1, rs.len());
    ///     let rs = nlp.ner(&vec!["成都商报记者 姚永忠".to_owned(), "微软XP操作系统今日正式退休".to_owned()], 2, false).unwrap();
    ///     assert_eq!(2, rs.len());
    /// }
    /// ```
    pub fn ner(&self, contents: &[String], sensitivity: usize, segmented: bool) -> Result<Vec<NamedEntity>> {
        let data = contents.to_json();
        let sensitivity_str = sensitivity.to_string();
        let params = match segmented {
            true => vec![("sensitivity", sensitivity_str.as_ref()), ("segmented", "1")],
            false => vec![("sensitivity", sensitivity_str.as_ref())],
        };
        self.post::<Vec<NamedEntity>>("/ner/analysis", params, &data)
    }

    /// [分词与词性标注接口](http://docs.bosonnlp.com/tag.html)
    ///
    /// ``contents``: 需要做分词与词性标注的文本序列
    ///
    /// ``space_mode``: 空格保留选项，0-3 有效
    ///
    /// ``oov_level``: 枚举强度选项，0-4 有效
    ///
    /// ``t2s``: 是否开启繁体转简体
    ///
    /// ``special_char_conv``: 是否转化特殊字符，针对回车、Tab 等特殊字符。
    ///
    /// # 使用示例
    ///
    /// ```
    /// extern crate bosonnlp;
    ///
    /// use bosonnlp::BosonNLP;
    ///
    /// fn main() {
    ///     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
    ///     let rs = nlp.tag(&vec!["成都商报记者 姚永忠".to_owned()], 0, 3, false, false).unwrap();
    ///     assert_eq!(1, rs.len());
    /// }
    /// ```
    pub fn tag(&self,
               contents: &[String],
               space_mode: usize,
               oov_level: usize,
               t2s: bool,
               special_char_conv: bool)
               -> Result<Vec<Tag>> {
        let data = contents.to_json();
        let t2s_str = match t2s {
            true => "1",
            false => "0",
        };
        let special_char_conv_str = match special_char_conv {
            true => "1",
            false => "0",
        };
        let space_mode_str = space_mode.to_string();
        let oov_level_str = oov_level.to_string();
        let params = vec![
            ("space_mode", space_mode_str.as_ref()),
            ("oov_level", oov_level_str.as_ref()),
            ("t2s", t2s_str),
            ("special_char_conv", special_char_conv_str),
        ];
        self.post("/tag/analysis", params, &data)
    }

    /// [新闻摘要接口](http://docs.bosonnlp.com/summary.html)
    ///
    /// ``title``: 需要做摘要的新闻标题，如果没有则传入空字符串
    ///
    /// ``content``: 需要做摘要的新闻正文
    ///
    /// ``word_limit``: 摘要字数限制
    ///
    /// ``not_exceed``: 是否严格限制字数
    ///
    /// # 使用示例
    ///
    /// ```
    /// extern crate bosonnlp;
    ///
    /// use bosonnlp::BosonNLP;
    ///
    /// fn main() {
    ///     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
    ///     let title = "前优酷土豆技术副总裁黄冬加盟芒果TV任CTO";
    ///     let content = "腾讯科技讯（刘亚澜）10月22日消息，前优酷土豆技术副总裁黄冬已于日前正式加盟芒果TV，出任CTO一职。";
    ///     let rs = nlp.summary(title, content, 1.0, false);
    ///     assert!(rs.is_ok());
    /// }
    /// ```
    pub fn summary<T: Into<String>>(&self, title: T, content: T, word_limit: f32, not_exceed: bool) -> Result<String> {
        let data = jsonway::object(|obj| {
                       obj.set("title", title.into());
                       obj.set("content", content.into());
                       obj.set("percentage", word_limit);
                       if not_exceed {
                           obj.set("not_exceed", 1);
                       } else {
                           obj.set("not_exceed", 0);
                       }
                   })
                       .unwrap();
        self.post::<String>("/summary/analysis", vec![], &data)
    }

    /// [文本聚类接口](http://docs.bosonnlp.com/cluster.html)
    ///
    /// ``task_id``: 唯一的 task_id，话题聚类任务的名字，可由字母和数字组成
    ///
    /// ``alpha``: 聚类最大 cluster 大小，一般为 0.8
    ///
    /// ``beta``: 聚类平均 cluster 大小，一般为 0.45
    ///
    /// ``timeout``: 等待文本聚类任务完成的秒数，一般为 1800 秒
    ///
    /// # 使用示例
    ///
    /// ```
    /// extern crate bosonnlp;
    ///
    /// use bosonnlp::{BosonNLP, ClusterContent};
    ///
    /// fn main() {
    ///     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
    ///     let contents = vec![
    ///         ClusterContent::from("今天天气好"),
    ///         ClusterContent::from("今天天气好"),
    ///         ClusterContent::from("今天天气不错"),
    ///         ClusterContent::from("点点楼头细雨"),
    ///         ClusterContent::from("重重江外平湖"),
    ///         ClusterContent::from("当年戏马会东徐"),
    ///         ClusterContent::from("今日凄凉南浦"),
    ///     ];
    ///     let rs = nlp.cluster(&vec![], None, 0.8, 0.45, Some(10)).unwrap();
    ///     assert_eq!(0, rs.len());
    ///     let rs = nlp.cluster(&contents, None, 0.8, 0.45, Some(10)).unwrap();
    ///     assert_eq!(1, rs.len());
    /// }
    /// ```
    pub fn cluster(&self,
                   contents: &[ClusterContent],
                   task_id: Option<&str>,
                   alpha: f32,
                   beta: f32,
                   timeout: Option<u64>)
                   -> Result<Vec<TextCluster>> {
        let mut task = match task_id {
            Some(_id) => ClusterTask::new(self, _id),
            None => {
                let _id = Uuid::new_v4().to_simple_string();
                ClusterTask::new(self, _id)
            }
        };
        if !try!(task.push(contents)) {
            return Ok(vec![]);
        }
        try!(task.analysis(alpha, beta));
        try!(task.wait(timeout));
        let result = try!(task.result());
        try!(task.clear());
        Ok(result)
    }

    /// [典型意见接口](http://docs.bosonnlp.com/comments.html)
    ///
    /// ``task_id``: 唯一的 task_id，典型意见任务的名字，可由字母和数字组成
    ///
    /// ``alpha``: 聚类最大 cluster 大小，一般为 0.8
    ///
    /// ``beta``: 聚类平均 cluster 大小，一般为 0.45
    ///
    /// ``timeout``: 等待典型意见任务完成的秒数，一般为 1800 秒
    ///
    /// # 使用示例
    ///
    /// ```
    /// extern crate bosonnlp;
    ///
    /// use bosonnlp::{BosonNLP, ClusterContent};
    ///
    /// fn main() {
    ///     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
    ///     let contents = vec![
    ///         ClusterContent::from("今天天气好"),
    ///         ClusterContent::from("今天天气好"),
    ///         ClusterContent::from("今天天气不错"),
    ///         ClusterContent::from("点点楼头细雨"),
    ///         ClusterContent::from("重重江外平湖"),
    ///         ClusterContent::from("当年戏马会东徐"),
    ///         ClusterContent::from("今日凄凉南浦"),
    ///         ClusterContent::from("今天天气好"),
    ///         ClusterContent::from("今天天气好"),
    ///         ClusterContent::from("今天天气不错"),
    ///         ClusterContent::from("点点楼头细雨"),
    ///         ClusterContent::from("重重江外平湖"),
    ///         ClusterContent::from("当年戏马会东徐"),
    ///         ClusterContent::from("今日凄凉南浦"),
    ///     ];
    ///     let rs = nlp.comments(&vec![], None, 0.8, 0.45, Some(10)).unwrap();
    ///     assert_eq!(0, rs.len());
    ///     let rs = nlp.comments(&contents, None, 0.8, 0.45, Some(10)).unwrap();
    ///     assert_eq!(4, rs.len());
    /// }
    /// ```
    pub fn comments(&self,
                    contents: &[ClusterContent],
                    task_id: Option<&str>,
                    alpha: f32,
                    beta: f32,
                    timeout: Option<u64>)
                    -> Result<Vec<CommentsCluster>> {
        let mut task = match task_id {
            Some(_id) => CommentsTask::new(self, _id),
            None => {
                let _id = Uuid::new_v4().to_simple_string();
                CommentsTask::new(self, _id)
            }
        };
        if !try!(task.push(contents)) {
            return Ok(vec![]);
        }
        try!(task.analysis(alpha, beta));
        try!(task.wait(timeout));
        let result = try!(task.result());
        try!(task.clear());
        Ok(result)
    }
}
