//! BosonNLP 部分 API 输入/返回类型
use jsonway;
use uuid::Uuid;
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
    /// 命名实体结果
    pub entity: Vec<(usize, usize, String)>,
    /// 词性标注结果
    pub tag: Vec<String>,
    /// 分词结果
    pub word: Vec<String>,
}

/// 词性标注
#[derive(Debug, RustcDecodable, Clone)]
pub struct Tag {
    /// 词性标注结果
    pub tag: Vec<String>,
    /// 分词结果
    pub word: Vec<String>,
}

/// 文本聚类
#[derive(Debug, RustcDecodable, Clone)]
pub struct TextCluster {
    /// 该 cluster 最具代表性的文档
    pub _id: String,
    /// 所有属于该 cluster 的文档 ``_id``
    pub list: Vec<String>,
    /// 该 cluster 包含的文档数目
    pub num: usize,
}

/// 典型意见
#[derive(Debug, RustcDecodable, Clone)]
pub struct CommentsCluster {
    /// 该典型意见的标示
    pub _id: usize,
    /// 所有属于该典型意见的评论
    pub list: Vec<(String, String)>,
    /// 该典型意见类似的意见个数
    pub num: usize,
    /// 典型意见文本
    pub opinion: String,
}

/// 聚类任务状态
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TaskStatus {
    /// 成功接收到分析请求
    Received,
    /// 数据分析正在进行中
    Running,
    /// 分析已完成
    Done,
    /// 分析遇到错误退出
    Error,
}

/// 聚类任务单个输入内容
#[derive(Debug, Clone, RustcEncodable, PartialEq, Eq, Hash)]
pub struct ClusterContent {
    /// 文档编号
    pub _id: String,
    /// 文档内容
    pub text: String,
}

impl ToJson for ClusterContent {
    fn to_json(&self) -> Json {
        jsonway::object(|obj| {
            obj.set("_id", self._id.clone());
            obj.set("text", self.text.clone());
        })
            .unwrap()
    }
}

impl From<String> for ClusterContent {
    fn from(content: String) -> ClusterContent {
        ClusterContent {
            _id: Uuid::new_v4().to_simple_string(),
            text: content,
        }
    }
}

impl<'a> From<&'a str> for ClusterContent {
    fn from(content: &'a str) -> ClusterContent {
        ClusterContent {
            _id: Uuid::new_v4().to_simple_string(),
            text: String::from(content),
        }
    }
}

/// 将其他类型转换成聚类需要的数据类型
pub trait IntoClusterInput {
    fn into_cluster_input(self) -> Vec<ClusterContent>;
}


impl<T: Into<ClusterContent>> IntoClusterInput for Vec<T> {
    fn into_cluster_input(self) -> Vec<ClusterContent> {
        let mut ret = vec![];
        for item in self {
            ret.push(item.into());
        }
        ret
    }
}

impl<T: Into<String>> IntoClusterInput for Vec<(T, T)> {
    fn into_cluster_input(self) -> Vec<ClusterContent> {
        let mut ret = vec![];
        for item in self {
            ret.push(ClusterContent {
                _id: item.0.into(),
                text: item.1.into(),
            });
        }
        ret
    }
}
