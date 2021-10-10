extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[macro_export]
macro_rules! warn {
    ( $( $t:tt )* ) => {
        web_sys::console::warn_1(&format!( $( $t )* ).into());
    }
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! unwrap_or_log_and_return {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => {
                log!("{:#?}", e);
                return;
            }
        }
    };
}
