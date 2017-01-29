use hyper;
use hyper::status::StatusCode;
use serde_json;

error_chain! {
    types {
        Error, ErrorKind, ChainErr, Result;
    }

    links { }

    foreign_links {
        Io(::std::io::Error);
        Http(hyper::Error);
        Json(serde_json::Error);
    }

    errors {
        /// API 错误
        Api { code: StatusCode, reason: String } {
            description("API error")
            display("API error, code {}, reason {}", code, reason)
        }
        /// 聚类任务未找到
        TaskNotFound(task_id: String) {
            description("cluster task not found")
            display("cluster {} not found", task_id)
        }
        /// 聚类任务超时
        Timeout(task_id: String) {
            description("cluster task timed out")
            display("cluster {} timed out", task_id)
        }
    }
}
