use std::io::{Read, Write};
use std::iter::FromIterator;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{self, Value, Map};
use url::Url;
use uuid::Uuid;
use flate2::Compression;
use flate2::write::GzEncoder;
use reqwest::{mime, Client, Method};
use reqwest::header::{UserAgent, Accept, ContentLength, ContentType, ContentEncoding, Encoding, qitem};

use errors::*;
use rep::{Dependency, NamedEntity, Tag, TextCluster, CommentsCluster, ConvertedTime, ClusterContent};
use task::{ClusterTask, CommentsTask, Task};


/// 默认的 `BosonNLP` API 服务器地址
const DEFAULT_BOSONNLP_URL: &'static str = "http://api.bosonnlp.com";

/// `BosonNLP` API 鉴权 HTTP Header
header! { (XToken, "X-Token") => [String] }

/// [`BosonNLP`](http://bosonnlp.com) REST API 访问的封装
#[derive(Debug, Clone)]
pub struct BosonNLP {
    /// 用于 API 鉴权的 API Token
    pub token: String,
    /// 是否压缩大于 10K 的请求体，默认为 true
    pub compress: bool,
    /// `BosonNLP` HTTP API 的 URL，默认为 `http://api.bosonnlp.com`
    bosonnlp_url: String,
    /// hyper http Client
    client: Client,
}

impl Default for BosonNLP {
    fn default() -> BosonNLP {
        BosonNLP {
            token: "".to_string(),
            compress: true,
            bosonnlp_url: DEFAULT_BOSONNLP_URL.to_owned(),
            client: Client::new().expect("Error construct HTTP client"),
        }
    }
}

impl BosonNLP {
    /// 初始化一个新的 `BosonNLP` 实例
    pub fn new<T: Into<String>>(token: T) -> BosonNLP {
        BosonNLP {
            token: token.into(),
            ..Default::default()
        }
    }

    /// 使用自定义参数初始化一个新的 ``BosonNLP`` 实例
    pub fn with_options<T: Into<String>>(token: T, bosonnlp_url: T, compress: bool) -> BosonNLP {
        BosonNLP {
            token: token.into(),
            compress: compress,
            bosonnlp_url: bosonnlp_url.into(),
            ..Default::default()
        }
    }

    /// 使用自定义的 reqwest Client 初始化一个新的 ``BosonNLP`` 实例
    pub fn with_client<T: Into<String>>(token: T, client: Client) -> BosonNLP {
        BosonNLP {
            token: token.into(),
            client: client,
            ..Default::default()
        }
    }

    fn request<D, E>(&self, method: Method, endpoint: &str, params: Vec<(&str, &str)>, data: &E) -> Result<D>
    where
        D: DeserializeOwned,
        E: Serialize,
    {
        let url_string = format!("{}{}", self.bosonnlp_url, endpoint);
        let mut url = Url::parse(&url_string).unwrap();
        url.query_pairs_mut().extend_pairs(params.into_iter());
        let mut req = self.client.request(method.clone(), url)?;
        let req = req.header(UserAgent::new(
                format!("bosonnlp-rs/{}", env!("CARGO_PKG_VERSION")),
            ))
            .header(Accept(vec![
                qitem(mime::APPLICATION_JSON),
            ]))
            .header(XToken(self.token.clone()));
        let mut res = if method == Method::Post {
            let req = req.header(ContentType::json());
            let body = match serde_json::to_string(data) {
                Ok(d) => d,
                Err(..) => "".to_owned(),
            };
            if self.compress && body.len() > 10240 {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::Default);
                encoder.write_all(body.as_bytes())?;
                let compressed = encoder.finish()?;
                let req = req.header(ContentEncoding(vec![Encoding::Gzip]));
                req.body(compressed).send()?
            } else {
                req.body(body).send()?
            }
        } else {
            req.send()?
        };
        let mut body = match res.headers().get::<ContentLength>() {
            Some(&ContentLength(len)) => String::with_capacity(len as usize),
            _ => String::new(),
        };
        res.read_to_string(&mut body)?;
        let status = res.status();
        if !status.is_success() {
            let result: Value = match serde_json::from_str(&body) {
                Ok(obj) => obj,
                Err(..) => Value::Object(Map::new()),
            };
            let message = match result.get("message") {
                Some(msg) => msg.as_str().unwrap_or("").to_owned(),
                None => body,
            };
            return Err(
                (ErrorKind::Api {
                     code: status,
                     reason: message,
                 }).into(),
            );
        }
        Ok(serde_json::from_str::<D>(&body)?)
    }

    pub(crate) fn get<D>(&self, endpoint: &str, params: Vec<(&str, &str)>) -> Result<D>
    where
        D: DeserializeOwned,
    {
        self.request(Method::Get, endpoint, params, &Value::Null)
    }

    pub(crate) fn post<D, E>(&self, endpoint: &str, params: Vec<(&str, &str)>, data: &E) -> Result<D>
    where
        D: DeserializeOwned,
        E: Serialize,
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
    ///     let rs = nlp.sentiment(&["这家味道还不错"], "food").unwrap();
    ///     assert_eq!(1, rs.len());
    /// }
    /// ```
    pub fn sentiment<T: AsRef<str>>(&self, contents: &[T], model: &str) -> Result<Vec<(f32, f32)>> {
        let endpoint = format!("/sentiment/analysis?{}", model);
        let data = contents.iter().map(|c| c.as_ref()).collect::<Vec<_>>();
        self.post(&endpoint, vec![], &data)
    }

    /// [时间转换接口](http://docs.bosonnlp.com/time.html)
    ///
    /// ``content``: 需要做时间转换的文本
    ///
    /// ``basetime``: 时间描述时的基准时间戳。如果为 ``None`` ，使用服务器当前的GMT+8时间
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
    ///     let time = nlp.convert_time("2013年二月二十八日下午四点三十分二十九秒", None).unwrap();
    ///     assert_eq!("2013-02-28 16:30:29", &time.timestamp.unwrap());
    ///     assert_eq!("timestamp", &time.format);
    /// }
    /// ```
    pub fn convert_time<T: AsRef<str>>(&self, content: T, basetime: Option<T>) -> Result<ConvertedTime> {
        if let Some(base) = basetime {
            let params = vec![("pattern", content.as_ref()), ("basetime", base.as_ref())];
            return self.post("/time/analysis", params, &Value::Null);
        } else {
            let params = vec![("pattern", content.as_ref())];
            return self.post("/time/analysis", params, &Value::Null);
        };
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
    ///     let rs = nlp.classify(&["俄否决安理会谴责叙军战机空袭阿勒颇平民"]).unwrap();
    ///     assert_eq!(vec![5usize], rs);
    /// }
    /// ```
    pub fn classify<T: AsRef<str>>(&self, contents: &[T]) -> Result<Vec<usize>> {
        let data = contents.iter().map(|c| c.as_ref()).collect::<Vec<_>>();
        self.post("/classify/analysis", vec![], &data)
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
        self.post(
            "/suggest/analysis",
            vec![("top_k", &top_k.to_string())],
            &word.as_ref(),
        )
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
        let top_k_str = top_k.to_string();
        let params = if segmented {
            vec![("top_k", top_k_str.as_ref()), ("segmented", "1")]
        } else {
            vec![("top_k", top_k_str.as_ref())]
        };
        self.post("/keywords/analysis", params, &text.as_ref())
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
    ///     let rs = nlp.depparser(&["今天天气好"]).unwrap();
    ///     assert_eq!(1, rs.len());
    ///     let dep0 = &rs[0];
    ///     assert_eq!(vec![2isize, 2isize, -1isize], dep0.head);
    ///     let rs = nlp.depparser(&["今天天气好", "美好的世界"]).unwrap();
    ///     assert_eq!(2, rs.len());
    /// }
    /// ```
    pub fn depparser<T: AsRef<str>>(&self, contents: &[T]) -> Result<Vec<Dependency>> {
        let data = contents.iter().map(|c| c.as_ref()).collect::<Vec<_>>();
        self.post("/depparser/analysis", vec![], &data)
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
    ///     let rs = nlp.ner(&["成都商报记者 姚永忠"], 2, false).unwrap();
    ///     assert_eq!(1, rs.len());
    ///     let rs = nlp.ner(&["成都商报记者 姚永忠", "微软XP操作系统今日正式退休"], 2, false).unwrap();
    ///     assert_eq!(2, rs.len());
    /// }
    /// ```
    pub fn ner<T: AsRef<str>>(&self, contents: &[T], sensitivity: usize, segmented: bool) -> Result<Vec<NamedEntity>> {
        let data = contents.iter().map(|c| c.as_ref()).collect::<Vec<_>>();
        let sensitivity_str = sensitivity.to_string();
        let params = if segmented {
            vec![
                ("sensitivity", sensitivity_str.as_ref()),
                ("segmented", "1"),
            ]
        } else {
            vec![("sensitivity", sensitivity_str.as_ref())]
        };
        self.post("/ner/analysis", params, &data)
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
    ///     let rs = nlp.tag(&["成都商报记者 姚永忠"], 0, 3, false, false).unwrap();
    ///     assert_eq!(1, rs.len());
    /// }
    /// ```
    pub fn tag<T: AsRef<str>>(
        &self,
        contents: &[T],
        space_mode: usize,
        oov_level: usize,
        t2s: bool,
        special_char_conv: bool,
    ) -> Result<Vec<Tag>> {
        let data = contents.iter().map(|c| c.as_ref()).collect::<Vec<_>>();
        let t2s_str = if t2s { "1" } else { "0" };
        let special_char_conv_str = if special_char_conv { "1" } else { "0" };
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
        let not_exceed = if not_exceed { 1 } else { 0 };
        let data = json!({
            "title": title.into(),
            "content": content.into(),
            "percentage": word_limit,
            "not_exceed": not_exceed
        });
        self.post("/summary/analysis", vec![], &data)
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
    /// use bosonnlp::BosonNLP;
    ///
    /// fn main() {
    ///     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
    ///     let contents = vec![
    ///         "今天天气好",
    ///         "今天天气好",
    ///         "今天天气不错",
    ///         "点点楼头细雨",
    ///         "重重江外平湖",
    ///         "当年戏马会东徐",
    ///         "今日凄凉南浦",
    ///     ];
    ///     let rs = nlp.cluster(&contents, None, 0.8, 0.45, Some(10)).unwrap();
    ///     assert_eq!(1, rs.len());
    /// }
    /// ```
    pub fn cluster<T: AsRef<str>>(
        &self,
        contents: &[T],
        task_id: Option<&str>,
        alpha: f32,
        beta: f32,
        timeout: Option<u64>,
    ) -> Result<Vec<TextCluster>> {
        let mut task = match task_id {
            Some(_id) => ClusterTask::new(self, _id),
            None => {
                let _id = Uuid::new_v4().simple().to_string();
                ClusterTask::new(self, _id)
            }
        };
        let tasks: Vec<ClusterContent> = Vec::from_iter(contents.iter().map(|c| c.into()));
        if !task.push(&tasks)? {
            return Ok(vec![]);
        }
        task.analysis(alpha, beta)?;
        task.wait(timeout)?;
        let result = task.result()?;
        task.clear()?;
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
    /// use bosonnlp::BosonNLP;
    ///
    /// fn main() {
    ///     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
    ///     let contents = vec![
    ///         "今天天气好",
    ///         "今天天气好",
    ///         "今天天气不错",
    ///         "点点楼头细雨",
    ///         "重重江外平湖",
    ///         "当年戏马会东徐",
    ///         "今日凄凉南浦",
    ///         "今天天气好",
    ///         "今天天气好",
    ///         "今天天气不错",
    ///         "点点楼头细雨",
    ///         "重重江外平湖",
    ///         "当年戏马会东徐",
    ///         "今日凄凉南浦",
    ///     ];
    ///     let rs = nlp.comments(&contents, None, 0.8, 0.45, Some(10)).unwrap();
    ///     assert_eq!(4, rs.len());
    /// }
    /// ```
    pub fn comments<T: AsRef<str>>(
        &self,
        contents: &[T],
        task_id: Option<&str>,
        alpha: f32,
        beta: f32,
        timeout: Option<u64>,
    ) -> Result<Vec<CommentsCluster>> {
        let mut task = match task_id {
            Some(_id) => CommentsTask::new(self, _id),
            None => {
                let _id = Uuid::new_v4().simple().to_string();
                CommentsTask::new(self, _id)
            }
        };
        let tasks: Vec<ClusterContent> = Vec::from_iter(contents.iter().map(|c| c.into()));
        if !task.push(&tasks)? {
            return Ok(vec![]);
        }
        task.analysis(alpha, beta)?;
        task.wait(timeout)?;
        let result = task.result()?;
        task.clear()?;
        Ok(result)
    }
}
