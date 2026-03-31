pub fn handle_device_tree(addr: usize) {
    use core::ptr::slice_from_raw_parts;

    println!("DTC addr: 0x{:x}", addr);
    let dtc_bytes = unsafe { &*slice_from_raw_parts(addr as *const u8, usize::MAX) };
    let devtree = unsafe {
        // Get the actual size of the device tree after reading its header.
        let size = DevTree::read_totalsize(&dtc_bytes).unwrap();
        print!("Device tree size: {}\n", size);
        let buf = &dtc_bytes[..size];

        // Create the device tree handle
        DevTree::new(buf).unwrap()
    };

    use fdt_rs::base::*;
    use fdt_rs::prelude::*;

    //   RAM : ORIGIN = 0x82000000, LENGTH = 16M

    // let mut iii = devtree.nodes();
    //
    // while let Some(node) = iii.next().unwrap() {
    //     println!("Node: {}", node.name().unwrap());
    // }

    // Iterate through all "ns16550a" compatible nodes within the device tree.
    // If found, print the name of each node (including unit address).
    let mut node_iter = devtree.nodes();
    while let Some(node) = node_iter.next().unwrap() {
        if node.name().unwrap_or("") == "memory@80000000" {
            for p in node.props().iterator() {
                if let Ok(pp) = p {
                    println!(
                        "{} {:x}",
                        pp.name().unwrap_or("errr"),
                        pp.u64(0).unwrap_or(32)
                    );
                } else {
                    println!("Errorrr");
                }
            }
        }
    }
}
