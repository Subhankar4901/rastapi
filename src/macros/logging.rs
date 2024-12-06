extern crate chrono;
#[macro_export]
macro_rules! log_info {
    ($msg:expr) => {
        {

            let now=chrono::Local::now();
            println!("\x1b[1;34m[INFO]\x1b[0m [{}] {}", now.format("%Y-%m-%d %H:%M:%S"), $msg);
        }
    };
    ($fmt:expr,$($arg:tt)*)=>{
        {
            let now=chrono::Local::now();
            println!("\x1b[1;34m[INFO]\x1b[0m [{}] {}",now.format("%Y-%m-%d %H:%M:%S"),format!($fmt, $($arg)*));
        }
    }
}
#[macro_export]
macro_rules! log_warning {
    ($msg:expr) => {{
        let now = chrono::Local::now();
        println!(
            "\x1b[1;33m[WARNING]\x1b[0m [{}] {}",
            now.format("%Y-%m-%d %H:%M:%S"),
            $msg
        );
    }};
}
// #[macro_export]
// macro_rules! log_error {
//     ($msg:expr) => {
//         {
//             let now=chrono::Local::now();
//             println!("\x1b[31m[ERROR]\x1b[0m [{}] {}", now.format("%Y-%m-%d %H:%M:%S"), $msg);
//         }
//     };
// }
#[macro_export]
macro_rules! error {
    () => {
        {
            let now=chrono::Local::now();
            println!("\x1b[1;31m[ERROR]\x1b[0m [{}] [{}:{}] :",now.format("%Y-%m-%d %H:%M:%S"),file!(),line!());
        }
    };
    ($msg:expr)=>{
        {
            let now=chrono::Local::now();
            println!("\x1b[1;31m[ERROR]\x1b[0m [{}] [{}:{}] :",now.format("%Y-%m-%d %H:%M:%S"),file!(),line!());
            println!("Message : {}",$msg);
        }
    };
    ($fmt:expr,$($arg:tt)*)=>{
        {
            let now=chrono::Local::now();
            println!("\x1b[1;31m[ERROR]\x1b[0m [{}] [{}:{}] :",now.format("%Y-%m-%d %H:%M:%S"),file!(),line!());
            println!("Message : {}",format!($fmt, $($arg)*));
        }
    }
}
#[macro_export]
macro_rules! log_response_4xx {
    ($method:expr,$rsrc:expr,$status:expr,$msg:expr) => {{
        let now = chrono::Local::now();
        println!(
            "\x1b[1;34m[{}]\x1b[0m [{}] - {} - \x1b[1;31m{}\x1b[0m - {}",
            $method,
            now.format("%Y-%m-%d %H:%M:%S"),
            $rsrc,
            $status,
            $msg
        );
    }};
    ($status:expr) => {{
        let now = chrono::Local::now();
        println!(
            "\x1b[1m[{}]\x1b[0m - \x1b[1;31m{}\x1b[0m",
            now.format("%Y-%m-%d %H:%M:%S"),
            $status
        );
    }};
}
#[macro_export]
macro_rules! log_response_2xx_content {
    ($method:expr,$rsrc:expr,$status:expr,$contype:expr) => {
        let now = chrono::Local::now();
        println!(
            "\x1b[1;34m[{}]\x1b[0m [{}] - {} - \x1b[1;32m{}\x1b[0m - {} Response",
            $method,
            now.format("%Y-%m-%d %H:%M:%S"),
            $rsrc,
            $status,
            $contype
        );
    };
}
#[macro_export]
macro_rules! log_response_2xx_file {
    ($method:expr,$rsrc:expr,$status:expr,$loc:expr) => {
        let now = chrono::Local::now();
        println!(
            "\x1b[1;34m[{}]\x1b[0m [{}] - {} - \x1b[1;32m{}\x1b[0m - Response file : {:?}",
            $method,
            now.format("%Y-%m-%d %H:%M:%S"),
            $rsrc,
            $status,
            $loc
        );
    };
}
#[macro_export]
macro_rules! log_response_3xx_content {
    ($method:expr,$rsrc:expr,$status:expr,$contype:expr) => {
        let now = chrono::Local::now();
        println!(
            "\x1b[1;34m[{}]\x1b[0m [{}] - {} - \x1b[1;33m{}\x1b[0m - {} Response",
            $method,
            now.format("%Y-%m-%d %H:%M:%S"),
            $rsrc,
            $status,
            $contype
        );
    };
}
#[macro_export]
macro_rules! log_response_3xx_file {
    ($method:expr,$rsrc:expr,$status:expr,$loc:expr) => {
        let now = chrono::Local::now();
        println!(
            "\x1b[1;34m[{}]\x1b[0m [{}] - {} - \x1b[1;33m{}\x1b[0m - Response file : {:?}",
            $method,
            now.format("%Y-%m-%d %H:%M:%S"),
            $rsrc,
            $status,
            $loc
        );
    };
}
#[macro_export]
macro_rules! log_response_5xx {
    ($method:expr,$rsrc:expr,$status:expr) => {
        let now = chrono::Local::now();
        println!(
            "\x1b[1;34m[{}]\x1b[0m [{}] - {} - \x1b[1;31m{}\x1b[0m",
            $method,
            now.format("%Y-%m-%d %H:%M:%S"),
            $rsrc,
            $status
        );
    };
}
