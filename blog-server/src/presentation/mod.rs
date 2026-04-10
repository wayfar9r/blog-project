pub mod convert;
pub mod dto;
pub mod grpc_service;
pub mod http_handlers;
pub mod middleware;

const DEFAULT_LIST_OFFSET: u32 = 0;
const DEFAULT_LIST_LIMIT: u8 = 10;

pub mod exchange {
    tonic::include_proto!("proto.blog");
}
