use crate::devices::device_tree::DeviceTree;
use crate::devices::device_tree::Node;

pub enum DriverKind {
    BusDriver {
        rescan: fn (),
        do_smth: fn (usize, usize, u16),
    },
    DeviceDriver {
        foo: fn (),
    },
}

pub struct Driver {
    name: &'static str,
    attach: fn (node: Node),
    kind: DriverKind,
}
