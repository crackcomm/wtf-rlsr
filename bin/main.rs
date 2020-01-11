fn main() {
    if let Err(e) = wtf_rlsr::execute() {
        println!("error: {:?}", e);
    }
}
