//! BosonNLP 部分 API 输入/返回类型
use jsonway;
use rustc_serialize::json::{Json, ToJson};

/// 依存文法
#[derive(Debug, RustcDecodable, Clone)]
pub struct Dependency {
    pub head: Vec<isize>,
    pub role: Vec<String>,
    pub tag: Vec<String>,
    pub word: Vec<String>,
}

/// 命名实体
#[derive(Debug, RustcDecodable, Clone)]
pub struct NamedEntity {
    pub entity: Vec<(usize, usize, String)>,
    pub tag: Vec<String>,
    pub word: Vec<String>,
}

/// 词性标注
#[derive(Debug, RustcDecodable, Clone)]
pub struct Tag {
    pub tag: Vec<String>,
    pub word: Vec<String>,
}

/// 文本聚类
#[derive(Debug, RustcDecodable, Clone)]
pub struct TextCluster {
    pub _id: usize,
    pub list: Vec<usize>,
    pub num: usize,
}

/// 典型意见
#[derive(Debug, RustcDecodable, Clone)]
pub struct CommentsCluster {
    pub _id: usize,
    pub list: Vec<(String, usize)>,
    pub num: usize,
    pub opinion: String,
}

#[derive(Debug, RustcDecodable, Clone)]
pub struct TaskStatusResp {
    pub status: String,
}

/// 聚类任务状态
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TaskStatus {
    Received,
    Running,
    Done,
    Error,
}

#[derive(Debug, Clone, RustcEncodable)]
pub struct ClusterContent {
    pub _id: String,
    pub text: String,
}

impl ToJson for ClusterContent {
    fn to_json(&self) -> Json {
        jsonway::object(|obj| {
            obj.set("_id", self._id.clone());
            obj.set("text", self.text.clone());
        }).unwrap()
    }
}
