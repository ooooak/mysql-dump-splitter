fn sleep(sec: u64){
    println!("sleep");
    use std::{thread, time};
    
    let from_millis = time::Duration::from_millis(sec * 1000);
    // let now = time::Instant::now();
    thread::sleep(from_millis);
}

