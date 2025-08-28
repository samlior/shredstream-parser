use std::{
    net::SocketAddr,
    sync::{atomic::AtomicBool, Arc},
    thread::JoinHandle,
    time::Duration,
};

use crate::forwarder::DeshreddedEntry;
use crate::transaction::parse_transaction;
use crossbeam_channel::Receiver;
use log::{debug, info};
use tokio::sync::broadcast::{Receiver as BroadcastReceiver, Sender};
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use transaction_protos::transaction::{
    transaction_service_server::{
        TransactionService as PbTransactionService, TransactionServiceServer,
    },
    TransactionBatch,
};

#[derive(Debug)]
pub struct TransactionService {
    entry_sender: Arc<Sender<DeshreddedEntry>>,
}

pub fn start_server_thread(
    addr: SocketAddr,
    entry_sender: Arc<Sender<DeshreddedEntry>>,
    exit: Arc<AtomicBool>,
    shutdown_receiver: Receiver<()>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let server_handle = runtime.spawn(async move {
            info!("starting server on {:?}", addr);
            tonic::transport::Server::builder()
                .add_service(TransactionServiceServer::new(TransactionService {
                    entry_sender,
                }))
                .serve(addr)
                .await
                .unwrap();
        });

        while !exit.load(std::sync::atomic::Ordering::Relaxed) {
            if shutdown_receiver
                .recv_timeout(Duration::from_secs(1))
                .is_ok()
            {
                server_handle.abort();
                info!("shutting down entries server");
                break;
            }
        }
    })
}

#[tonic::async_trait]
impl PbTransactionService for TransactionService {
    type SubscribeTransactionsStream = ReceiverStream<Result<TransactionBatch, tonic::Status>>;

    async fn subscribe_transactions(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::SubscribeTransactionsStream>, tonic::Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let mut entry_receiver: BroadcastReceiver<DeshreddedEntry> = self.entry_sender.subscribe();

        tokio::spawn(async move {
            while let Ok(entry) = entry_receiver.recv().await {
                let mut transaction_batch = TransactionBatch {
                    slot: entry.slot,
                    transactions: vec![],
                };

                for entry in entry.entries {
                    for versioned_transaction in entry.transactions {
                        let transactions = parse_transaction(&versioned_transaction);
                        if !transactions.is_empty() {
                            transaction_batch.transactions.extend(transactions);
                        }
                    }
                }

                match tx.send(Ok(transaction_batch)).await {
                    Ok(_) => (),
                    Err(_e) => {
                        debug!("client disconnected");
                        break;
                    }
                }
            }
        });

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}
