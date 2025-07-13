use crate::println;
use core::arch::asm;
use fdt::Fdt;

pub fn dtb_ptr() -> usize {
    let dtb_ptr: usize;
    unsafe {
        asm!("mv {}, a1", out(reg) dtb_ptr);
    }
    dtb_ptr
}

/// Reads the 'reg' property based on the address and size cells
fn read_reg_from_cells(reg: &[u8], address_cells: u32, size_cells: u32) -> Option<(usize, usize)> {
    if reg.len() < (address_cells + size_cells) as usize {
        println!(
            "Error: 'reg' property is too short for address_cells={} and size_cells={}",
            address_cells, size_cells
        );
        return None;
    }
    let address = match address_cells {
        1 => u32::from_be_bytes(reg[0..4].try_into().unwrap()) as usize,
        2 => u64::from_be_bytes(reg[0..8].try_into().unwrap()) as usize,
        _ => {
            println!("Error: Unsupported address_cells value: {}", address_cells);
            return None;
        }
    };
    let size =
        match size_cells {
            1 => u32::from_be_bytes(reg[address_cells as usize * 4..][0..4].try_into().unwrap())
                as usize,
            2 => u64::from_be_bytes(reg[address_cells as usize * 4..][0..8].try_into().unwrap())
                as usize,
            _ => {
                println!("Error: Unsupported size_cells value: {}", size_cells);
                return None;
            }
        };
    Some((address, size))
}

pub fn init(dtb_ptr: usize) {
    println!("Initializing device tree...");
    println!("Device Tree Pointer: {:#x}", dtb_ptr);

    let dtb = unsafe { core::slice::from_raw_parts(dtb_ptr as *const u8, 0x10000) };
    let fdt = match Fdt::new(dtb) {
        Ok(fdt) => fdt,
        Err(e) => {
            println!("Failed to parse device tree: {:?}", e);

            #[allow(clippy::empty_loop)]
            loop {}
        }
    };

    println!("Device Model: {:?}", fdt.root().model());
    println!("Device Compatible: {:?}", fdt.root().compatible().first());
    println!(
        "Device Memory Regions: {:?}",
        fdt.memory().regions().count()
    );
    println!("Device CPU(s): {:?}", fdt.cpus().count());
    let soc = fdt.find_node("/soc");
    if let Some(soc) = soc {
        println!("SoC Node: {:?}", soc.name);

        // These are used to be able to parse the 'reg' property correctly
        let address_cells = soc
            .property("#address-cells")
            .map(|p| p.value)
            .map(|v| v.try_into().unwrap_or([0, 0, 0, 0]))
            .map(u32::from_be_bytes)
            .unwrap_or(0);
        let size_cells = soc
            .property("#size-cells")
            .map(|p| p.value)
            .map(|v| v.try_into().unwrap_or([0, 0, 0, 0]))
            .map(u32::from_be_bytes)
            .unwrap_or(0);

        println!("Address Cells: {}", address_cells);
        println!("Size Cells: {}", size_cells);

        for child in soc.children() {
            if child.name.split("@").next() == Some("serial") {
                println!("SOC Child Node: {:?}", child.name);
                if let Some(reg) = child.property("reg") {
                    let (base, _) =
                        read_reg_from_cells(reg.value, address_cells, size_cells).unwrap();
                    println!("\tSerial Node Base Register: 0x{:X}", base);
                } else {
                    println!("\nSerial Node has no 'reg' property");
                }
                if let Some(clock) = child.property("clock-frequency") {
                    let clock = u32::from_be_bytes(clock.value.try_into().unwrap());
                    println!("\tSerial Node Clock Frequency: {}", clock);
                } else {
                    println!("\nSerial Node has no 'clock-frequency' property");
                }
            } else {
                println!("SOC Node: {:?}", child.name);
                if let Some(reg) = child.property("reg") {
                    if let Some((base, size)) =
                        read_reg_from_cells(reg.value, address_cells, size_cells)
                    {
                        println!("\tNode Base Register: 0x{:X}, Size: 0x{:X}", base, size);
                    } else {
                        println!("\tNode has no valid 'reg' property");
                    }
                } else {
                    println!("\tNode has no 'reg' property");
                }
            }
        }
    } else {
        println!("No SoC node found in device tree");
    }

    println!("Device Tree Initialization complete");
}
