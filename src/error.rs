pub type UtcpResult<T> = Result<T, UtcpErr>;

#[derive(Debug)]
pub enum UtcpErr {
    Net(String),
}
