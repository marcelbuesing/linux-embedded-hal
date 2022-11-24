use embedded_hal::can::blocking::Can;
use linux_embedded_hal::{CanSocket, CanFrame};

fn main() {
    let mut socket = CanSocket::open("vcan0").expect("Failed to open socket");

    let tx_frame = CanFrame::new(0x123, &[0x11, 0x22, 0x33], false, false).expect("Invalid frame");

    // You can observe the transmition e.g. using canutil's candump:
    // candump vcan0
    socket.transmit(&tx_frame).expect("Failed to transmit");

    // Send a frame to vcan0 otherwise this will block e.g. using canutil's cansend:
    // cansend vcan0 456#445566
    let rx_frame = socket.receive().expect("Failed to transmit");
    println!("Read frame: {:?}", rx_frame);
}