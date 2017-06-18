use uuid::Uuid;

/// 依存文法
#[derive(Debug, Deserialize, Clone)]
pub struct Dependency {
    pub head: Vec<isize>,
    pub role: Vec<String>,
    pub tag: Vec<String>,
    pub word: Vec<String>,
}

/// 命名实体
#[derive(Debug, Deserialize, Clone)]
pub struct NamedEntity {
    /// 命名实体结果
    pub entity: Vec<(usize, usize, String)>,
    /// 词性标注结果
    pub tag: Vec<String>,
    /// 分词结果
    pub word: Vec<String>,
}

/// 词性标注
#[derive(Debug, Deserialize, Clone)]
pub struct Tag {
    /// 词性标注结果
    pub tag: Vec<String>,
    /// 分词结果
    pub word: Vec<String>,
}

/// 时间转换结果
#[derive(Debug, Deserialize, Clone)]
pub struct ConvertedTime {
    /// 时间点，ISO8601 格式的时间字符串
    pub timestamp: Option<String>,
    /// 时间量，格式为 "xday,HH:MM:SS" 或 "HH:MM:SS" 的字符串
    pub timedelta: Option<String>,
    /// 表示时间点组成的时间区间结果，格式为 ``(timestamp, timestamp)``
    ///  或 ``(timedelta, timedelta)`` 表示时间区间的起始和结束时间
    pub timespan: Option<(String, String)>,
    /// 时间数据格式, 有 ``timestamp``、``timedelta``、``timespan_0``、和 ``timespan_1``
    #[serde(rename = "type")]
    pub format: String,
}

/// 文本聚类
#[derive(Debug, Deserialize, Clone)]
pub struct TextCluster {
    /// 该 cluster 最具代表性的文档
    pub _id: String,
    /// 所有属于该 cluster 的文档 ``_id``
    pub list: Vec<String>,
    /// 该 cluster 包含的文档数目
    pub num: usize,
}

/// 典型意见
#[derive(Debug, Deserialize, Clone)]
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

/// 聚类任务提交响应
#[derive(Debug, Deserialize, Clone)]
pub struct TaskPushResp {
    pub task_id: String,
    pub count: usize,
}

/// 聚类任务状态响应
#[derive(Debug, Deserialize, Clone)]
pub struct TaskStatusResp {
    pub _id: String,
    pub status: String,
    pub count: usize,
}

/// 聚类任务单个输入内容
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct ClusterContent {
    /// 文档编号
    pub _id: String,
    /// 文档内容
    pub text: String,
}

impl From<String> for ClusterContent {
    fn from(content: String) -> ClusterContent {
        ClusterContent {
            _id: Uuid::new_v4().simple().to_string(),
            text: content,
        }
    }
}

impl<'a, T: ?Sized + AsRef<str>> From<&'a T> for ClusterContent {
    fn from(content: &'a T) -> ClusterContent {
        ClusterContent::from(content.as_ref().to_string())
    }
}
