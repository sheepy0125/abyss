pub mod index;

pub fn page_result_to_response(result: anyhow::Result<String>) -> windmark::response::Response {
    match result {
        Ok(res) => windmark::response::Response::success(res),
        Err(e) => windmark::response::Response::temporary_failure(format!("error! {e}")),
    }
}
