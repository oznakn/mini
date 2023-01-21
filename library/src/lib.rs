pub fn start() {
    println!("Hello, world from library!");
}

#[no_mangle]
pub extern "C" fn hello_world() {
    println!("Hello, world from C library!");
}
