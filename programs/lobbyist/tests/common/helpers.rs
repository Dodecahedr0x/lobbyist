#[macro_export]
macro_rules! assert_tx {
    ($tx_result:expr) => {
        match $tx_result {
            Ok(res) => res,
            Err(e) => {
                if !e.meta.logs.is_empty() {
                    eprintln!("Logs:\n{}", e.meta.logs.join("\n"));
                } else {
                    eprintln!("Error: {:?}", e);
                }
                panic!("Transaction failed")
            }
        }
    };
}
