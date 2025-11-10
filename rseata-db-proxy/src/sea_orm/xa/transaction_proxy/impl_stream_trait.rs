use crate::sea_orm::xa::transaction_proxy::{TransactionType, XATransactionProxy};
use sea_orm::{DbBackend, DbErr, QueryStream, Statement, StreamTrait, TransactionStream};
use std::pin::Pin;

// impl StreamTrait for XATransactionProxy {
//     type Stream<'a> = TransactionStream<'a>;
// 
//     fn get_database_backend(&self) -> DbBackend {
//         StreamTrait::get_database_backend(&self.xa_connection_proxy)
//     }
//     fn stream_raw<'a>(
//         &'a self,
//         stmt: Statement,
//     ) -> Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, DbErr>> + 'a + Send>> {
//         match &self.transaction_type {
//             TransactionType::Local(local) => {
//                 if let Some(local)=local.lock().unwrap().as_ref() {
//                   local.stream_raw(stmt)
//                 }else {
//                     unreachable!()
//                 } 
//             },
//             TransactionType::XA(_, ..) => {
//                 unreachable!()
//             }
//         }
//     }
// }
