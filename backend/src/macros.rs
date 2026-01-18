#[macro_export]
macro_rules! select_loop {
    ($($branches:tt)*) => {
        loop {
            tokio::select! {
                $($branches)*
            }
        }
    };
}
