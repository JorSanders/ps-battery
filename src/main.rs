mod ps5_bt; // your module file: ps5_bt.rs

fn main() {
    println!("Scanning for connected PS5 controllers...");
    ps5_bt::list_connected_ps5_controllers();
    println!("Scan complete.");
}
