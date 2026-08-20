use heapless::Vec;
use crate::devices::driver::Driver;

const SYSTEM_BUS_NAME: &'static str = "sysbus";

pub enum NodeKind {
    Bus {
        children: Vec<usize, 8>, 
    },
    Device,
}

pub struct BusInfo {
    bus_id: usize,
    child_id: usize,
}

pub struct DriverInfo {
    driver: Option<Driver>,
    device_id: usize,
}

pub struct Node {
    pub name: &'static str,
    pub global_id: usize,
    pub parent_bus: Option<BusInfo>,
    pub driver: Option<DriverInfo>,
    pub kind: NodeKind,
}

pub struct DeviceTree<const MAX_NODES: usize> {
    pub nodes: Vec<Node, MAX_NODES>,
}

impl<const MAX_NODES: usize> DeviceTree<MAX_NODES> {
    pub fn new() -> Self { Self { nodes: Vec::new() } }

    pub fn with_root(mut self) -> Result<Self, &'static str> {
        let root = Node {
            name: SYSTEM_BUS_NAME,
            global_id: 0,
            parent_bus: None,
            driver: None,
            kind: NodeKind::Bus { 
                children: Vec::new(),
            },
        };

        self.nodes.push(root).map_err(|_| "Tree is full")?;

        Ok(self)
    }

    pub fn add_node(
        &mut self,
        name: &'static str,
        bus_id: usize,
        child_id: usize,
        kind: NodeKind,
    ) -> Result<usize, &'static str> {
        let new_id = self.nodes.len();
        
        let new_node = Node {
            name: name,
            global_id: new_id,
            parent_bus: Some(BusInfo { bus_id, child_id }),
            driver: None,
            kind,
        };

        self.nodes.push(new_node).map_err(|_| "Tree is full")?;

        let parent_node = &mut self.nodes[bus_id];
        if let NodeKind::Bus { children, .. } = &mut parent_node.kind {
            children.push(new_id).map_err(|_| "Parent is full")?;
        }
        Ok(new_id)
    }
}

