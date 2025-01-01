use spam::data::system::System;

fn main() {
    let mut sys = System::new();
    sys.update_sys();
    sys.display();
}
