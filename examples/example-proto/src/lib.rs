pub mod client;
mod lazy;

pub mod rseata_proto {
  pub mod proto {
    tonic::include_proto!("cn.olbe.grpc.rust.examples.proto.user");
  }
}
