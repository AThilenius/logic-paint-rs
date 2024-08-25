extern crate web_sys;

// Note: the cfg target_arch is needed for local test paths, non-web targets will never be supported
// for the built binary (web-specific APIs are used heavily).

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!( $( $t )* ).into());

        #[cfg(not(target_arch = "wasm32"))]
        println!($( $t )*);
    }
}

#[macro_export]
macro_rules! warn {
    ( $( $t:tt )* ) => {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::warn_1(&format!( $( $t )* ).into());

        #[cfg(not(target_arch = "wasm32"))]
        println!($( $t )*);
    }
}

#[macro_export]
macro_rules! error {
    ( $( $t:tt )* ) => {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::error_1(&format!( $( $t )* ).into());

        #[cfg(not(target_arch = "wasm32"))]
        println!($( $t )*);
    }
}

#[macro_export]
macro_rules! unwrap_or_return {
    ($e:expr) => {
        match $e {
            Some(x) => x,
            None => {
                return;
            }
        }
    };
}

#[macro_export]
macro_rules! result_or_log_and_return {
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
