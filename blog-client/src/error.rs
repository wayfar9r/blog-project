#[derive(Debug, thiserror::Error)]
pub enum BlogClientError {
    #[error("http client error: {0}")]
    HttpClient(#[from] reqwest::Error),
    #[error("grpc client error: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),
    #[error("grpc error: {0}")]
    GrpcStatus(#[from] tonic::Status),
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("got unexpected status: {status}, message: {text}")]
    UnexpectedStatus { status: u16, text: String },
    #[error("unexpected client error: {0}")]
    ClientError(String),
}
