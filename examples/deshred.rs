use transaction_protos::transaction::transaction_service_client::TransactionServiceClient;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut client = TransactionServiceClient::connect("http://127.0.0.1:20010")
        .await
        .unwrap();
    let mut stream = client
        .subscribe_transactions(tonic::Request::new(()))
        .await
        .unwrap()
        .into_inner();

    while let Some(transacion_batch) = stream.message().await.unwrap() {
        for transaction in transacion_batch.transactions {
            println!(
                "slot {}, transaction: {:?}",
                transacion_batch.slot, transaction
            );
        }
    }

    Ok(())
}
