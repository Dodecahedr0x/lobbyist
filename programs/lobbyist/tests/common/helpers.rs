#[macro_export]
macro_rules! assert_tx {
    ($tx_result:expr) => {
        match $tx_result {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Logs:\n{}", e.meta.logs.join("\n"));
                panic!("Transaction failed")
            }
        }
    };
}
