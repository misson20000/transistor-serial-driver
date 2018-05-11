extern crate libusb;

use std::io::Write;

fn find_device<'a>(context: &'a libusb::Context) -> libusb::Result<libusb::Device<'a>> {
    for mut device in try!(context.devices()).iter() {
        let device_desc = try!(device.device_descriptor());

        if device_desc.vendor_id() == 0x1209 && device_desc.product_id() == 0x8b00 {
            return Ok(device);
        }
    }
    Err(libusb::Error::NotFound)
}

fn find_interface_descriptor<'a>(config: &'a libusb::ConfigDescriptor) -> libusb::Result<libusb::InterfaceDescriptor<'a>> {
    for interface in config.interfaces() {
        for descriptor in interface.descriptors() {
            if descriptor.class_code() == 0xFF && descriptor.sub_class_code() == 0x00 {
                return Ok(descriptor);
            }
        }
    }
    Err(libusb::Error::NotFound)
}


fn main() {
    let context = std::boxed::Box::new(libusb::Context::new().unwrap());
    let device = find_device(&context).expect("could not find serial console device");
    let config = device.active_config_descriptor().unwrap();
    let interface_descriptor = find_interface_descriptor(&config).expect("could not find serial console interface");
    
    let mut endp_iter = interface_descriptor.endpoint_descriptors();
    let endp_out = endp_iter.next().unwrap();
    let endp_in = endp_iter.next().unwrap();
    assert!(endp_iter.next().is_none());

    assert_eq!(endp_out.direction(), libusb::Direction::Out);
    assert_eq!(endp_in.direction(), libusb::Direction::In);
    assert_eq!(endp_out.transfer_type(), libusb::TransferType::Bulk);
    assert_eq!(endp_in.transfer_type(), libusb::TransferType::Bulk);
    
    println!("found serial console endpoints!");

    let mut handle = device.open().unwrap();
    if handle.kernel_driver_active(interface_descriptor.interface_number()).unwrap() {
        println!("detaching kernel driver for serial console");
        handle.detach_kernel_driver(interface_descriptor.interface_number()).unwrap();
    }
    handle.claim_interface(interface_descriptor.interface_number()).unwrap();
    println!("claimed serial console interface");
    
    loop {
        let buffer:&mut [u8; 0x8000] = &mut [0; 0x8000];
        let bytes_read = handle.read_bulk(endp_in.address(), buffer, std::time::Duration::new(10000, 0)).unwrap();
        std::io::stdout().write(&buffer[0..bytes_read]).unwrap();
    }
}
