use std::time::Duration;
use std::cmp::min;
use std::thread;

use super::BosonNLP;
use rep::{TextCluster, CommentsCluster, TaskStatus, ClusterContent, TaskPushResp, TaskStatusResp};
use errors::*;

/// 聚类任务属性
pub(crate) trait TaskProperty {
    /// 任务 ID
    fn task_id(&self) -> String;
}

/// 聚类任务
pub(crate) trait Task: TaskProperty {
    type Output;

    /// 批量上传需要处理的文本序列
    fn push(&mut self, contents: &[ClusterContent]) -> Result<bool>;
    /// 启动分析任务
    fn analysis(&self, alpha: f32, beta: f32) -> Result<()>;
    /// 获取任务状态
    fn status(&self) -> Result<TaskStatus>;
    /// 获取任务结果
    fn result(&self) -> Result<Self::Output>;
    /// 清空服务器端缓存的文本和结果
    fn clear(&self) -> Result<()>;

    /// 等待任务完成
    fn wait(&self, timeout: Option<u64>) -> Result<()> {
        let mut elapsed = Duration::from_secs(0u64);
        let mut seconds_to_sleep = Duration::from_secs(0u64);
        if let Some(_timeout) = timeout {
            seconds_to_sleep = min(seconds_to_sleep, Duration::from_secs(_timeout));
        }
        let mut i = 0usize;
        loop {
            thread::sleep(seconds_to_sleep);
            let status = self.status()?;
            if status == TaskStatus::Done {
                return Ok(());
            }
            elapsed += seconds_to_sleep;
            if let Some(_timeout) = timeout {
                if elapsed >= Duration::from_secs(_timeout) {
                    return Err(Error::Timeout(self.task_id()));
                }
            }
            i += 1usize;
            if i % 3usize == 0usize && seconds_to_sleep < Duration::from_secs(64u64) {
                seconds_to_sleep += seconds_to_sleep;
            }
        }
    }
}

/// 文本聚类任务
pub(crate) struct ClusterTask<'a> {
    task_id: String,
    contents: Vec<ClusterContent>,
    nlp: &'a BosonNLP,
}

impl<'a> ClusterTask<'a> {
    pub fn new<T: Into<String>>(nlp: &'a BosonNLP, task_id: T) -> ClusterTask<'a> {
        ClusterTask {
            task_id: task_id.into(),
            contents: vec![],
            nlp: nlp,
        }
    }
}

impl<'a> TaskProperty for ClusterTask<'a> {
    fn task_id(&self) -> String {
        self.task_id.clone()
    }
}

impl<'a> Task for ClusterTask<'a> {
    type Output = Vec<TextCluster>;

    /// 批量上传需要处理的文本序列
    fn push(&mut self, contents: &[ClusterContent]) -> Result<bool> {
        let endpoint = format!("/cluster/push/{}", self.task_id());
        if contents.is_empty() {
            return Ok(false);
        }
        for parts in contents.chunks(100) {
            let _: TaskPushResp = self.nlp.post(&endpoint, vec![], &parts)?;
            info!(
                "Pushed {} of {} documents for clustering",
                parts.len(),
                contents.len()
            );
        }
        self.contents.extend_from_slice(contents);
        Ok(true)
    }

    /// 启动分析任务
    fn analysis(&self, alpha: f32, beta: f32) -> Result<()> {
        let endpoint = format!("/cluster/analysis/{}", self.task_id());
        let alpha_str = alpha.to_string();
        let beta_str = beta.to_string();
        let params = vec![("alpha", alpha_str.as_ref()), ("beta", beta_str.as_ref())];
        let _: TaskStatusResp = self.nlp.get(&endpoint, params)?;
        info!("Cluster task {} analysis started", self.task_id());
        Ok(())
    }

    /// 获取任务状态
    fn status(&self) -> Result<TaskStatus> {
        let endpoint = format!("/cluster/status/{}", self.task_id());
        let status_resp: TaskStatusResp = self.nlp.get(&endpoint, vec![])?;
        let status_str = status_resp.status.to_lowercase();
        info!("Cluster task {} status: {}", self.task_id(), status_str);
        let ret = match status_str.as_ref() {
            "received" => TaskStatus::Received,
            "running" => TaskStatus::Running,
            "done" => TaskStatus::Done,
            "error" => TaskStatus::Error,
            "not found" => return Err(Error::TaskNotFound(self.task_id())),
            _ => unreachable!(),
        };
        Ok(ret)
    }

    /// 获取任务结果
    fn result(&self) -> Result<Vec<TextCluster>> {
        let endpoint = format!("/cluster/result/{}", self.task_id());
        self.nlp.get(&endpoint, vec![])
    }

    /// 清空服务器端缓存的文本和结果
    fn clear(&self) -> Result<()> {
        let endpoint = format!("/cluster/clear/{}", self.task_id());
        self.nlp
            .get::<String>(&endpoint, vec![])
            .unwrap_or_else(|_| "".to_owned());
        info!("Cluster task {} cleared", self.task_id());
        Ok(())
    }
}

/// 典型意见任务
pub(crate) struct CommentsTask<'a> {
    pub task_id: String,
    contents: Vec<ClusterContent>,
    nlp: &'a BosonNLP,
}

impl<'a> CommentsTask<'a> {
    pub fn new<T: Into<String>>(nlp: &'a BosonNLP, task_id: T) -> CommentsTask<'a> {
        CommentsTask {
            task_id: task_id.into(),
            contents: vec![],
            nlp: nlp,
        }
    }
}

impl<'a> TaskProperty for CommentsTask<'a> {
    fn task_id(&self) -> String {
        self.task_id.clone()
    }
}

impl<'a> Task for CommentsTask<'a> {
    type Output = Vec<CommentsCluster>;

    /// 批量上传需要处理的文本序列
    fn push(&mut self, contents: &[ClusterContent]) -> Result<bool> {
        let endpoint = format!("/comments/push/{}", self.task_id());
        if contents.is_empty() {
            return Ok(false);
        }
        for parts in contents.chunks(100) {
            let _: TaskPushResp = self.nlp.post(&endpoint, vec![], &parts)?;
            info!(
                "Pushed {} of {} documents for comments clustering",
                parts.len(),
                contents.len()
            );
        }
        self.contents.extend_from_slice(contents);
        Ok(true)
    }

    /// 启动分析任务
    fn analysis(&self, alpha: f32, beta: f32) -> Result<()> {
        let endpoint = format!("/comments/analysis/{}", self.task_id());
        let alpha_str = alpha.to_string();
        let beta_str = beta.to_string();
        let params = vec![("alpha", alpha_str.as_ref()), ("beta", beta_str.as_ref())];
        let _: TaskStatusResp = self.nlp.get(&endpoint, params)?;
        info!("Comments task {} analysis started", self.task_id());
        Ok(())
    }

    /// 获取任务状态
    fn status(&self) -> Result<TaskStatus> {
        let endpoint = format!("/comments/status/{}", self.task_id());
        let status_resp: TaskStatusResp = self.nlp.get(&endpoint, vec![])?;
        let status_str = status_resp.status.to_lowercase();
        info!("Comments task {} status: {}", self.task_id(), status_str);
        let ret = match status_str.as_ref() {
            "received" => TaskStatus::Received,
            "running" => TaskStatus::Running,
            "done" => TaskStatus::Done,
            "error" => TaskStatus::Error,
            "not found" => return Err(Error::TaskNotFound(self.task_id())),
            _ => unreachable!(),
        };
        Ok(ret)
    }

    /// 获取任务结果
    fn result(&self) -> Result<Vec<CommentsCluster>> {
        let endpoint = format!("/comments/result/{}", self.task_id());
        self.nlp.get(&endpoint, vec![])
    }

    /// 清空服务器端缓存的文本和结果
    fn clear(&self) -> Result<()> {
        let endpoint = format!("/comments/clear/{}", self.task_id());
        self.nlp
            .get::<String>(&endpoint, vec![])
            .unwrap_or_else(|_| "".to_owned());
        info!("Comments task {} cleared", self.task_id());
        Ok(())
    }
}
