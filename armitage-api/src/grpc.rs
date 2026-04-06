//! gRPC API implementation

pub fn create_grpc_server() -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
    tokio::spawn(async {
        // TODO: Implement gRPC service
        Ok(())
    })
}
