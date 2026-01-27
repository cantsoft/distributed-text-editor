#[macro_export]
macro_rules! select_loop {
    ($label:lifetime : $($branches:tt)*) => {
        $label: loop {
            tokio::select! {
                $($branches)*
            }
        }
    };
    ($($branches:tt)*) => {
        loop {
            tokio::select! {
                $($branches)*
            }
        }
    };
}
