pub type UtcpResult<T> = Result<T, UtcpErr>;

#[derive(Debug)]
pub enum UtcpErr {
    Net(String),
    Intr(String),
}

impl std::fmt::Display for UtcpErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UtcpErr::Net(s) => write!(f, "net error: {}", s),
            UtcpErr::Intr(s) => write!(f, "interrupt error: {}", s),
        }
    }
}
