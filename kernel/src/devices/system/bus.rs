use heapless::Vec;

pub struct SystemBusDeviceInfo {
    mem_start: usize,
    mem_end: usize,
}

static mut devs: Vec<SystemBusDevice, 16> = Vec::new();

pub fn get_data(child_id: usize) -> &SystemBusDeviceInfo {
    &devs[child_id]
}

