use std::ops;

/// TODO
#[derive(Debug)]
pub struct CanFrame(pub socketcan::CanFrame);

impl CanFrame {
    /// TODO
    pub fn new(id: u32, data: &[u8], rtr: bool, err: bool) -> Result<CanFrame, CanError> {
        let frame = socketcan::CanFrame::new(id, data, rtr, err).unwrap();
        Ok(CanFrame(frame))
    }
}

impl ops::Deref for CanFrame {
    type Target = socketcan::CanFrame;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for CanFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// TODO
pub struct CanSocket(pub socketcan::CanSocket);

impl CanSocket {
    /// TODO
    pub fn open(ifname: &str) -> Result<CanSocket, CanError> {
        // TODO fix error mapping
        let socket = socketcan::CanSocket::open(ifname).unwrap();
        Ok(CanSocket(socket))
    }
}

impl ops::Deref for CanSocket {
    type Target = socketcan::CanSocket;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for CanSocket {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// TODO
#[derive(Debug)]
pub struct CanError(pub socketcan::CanError);

impl ops::Deref for CanError {
    type Target = socketcan::CanError;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for CanError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

mod embedded_hal_impl {
    use super::*;
    use embedded_hal::can::{blocking, ExtendedId, StandardId};
    use embedded_hal::can::{Error, ErrorKind, Frame, Id};
    use socketcan::EFF_MASK;
    use std::convert::TryInto;
    use std::io;

    /// valid standard id bit (11bit)
    const SFF_MASK: u16 = 0x07ff;

    impl Error for CanError {
        fn kind(&self) -> ErrorKind {
            // TODO better output
            ErrorKind::Other
        }
    }

    impl blocking::Can for CanSocket {
        type Frame = CanFrame;
        type Error = CanError;
        fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
            self.write_frame_insist(frame)
                .map_err(|err| CanError { err })
        }
        fn receive(&mut self) -> Result<Self::Frame, Self::Error> {
            self.read_frame()
                .map(CanFrame)
                .map_err(|err| CanError { err })
        }
    }

    impl Frame for CanFrame {
        fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
            let raw_id = match id.into() {
                Id::Extended(extended_id) => extended_id.as_raw(),
                Id::Standard(standard_id) => standard_id.as_raw().into(),
            };
            let frame = socketcan::CanFrame::new(raw_id, data, false, false);
            frame.map(CanFrame).ok()
        }
        fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self> {
            // TODO fix unused dlc
            let raw_id = match id.into() {
                Id::Extended(extended_id) => extended_id.as_raw(),
                Id::Standard(standard_id) => standard_id.as_raw().into(),
            };
            let frame = socketcan::CanFrame::new(raw_id, &[], true, false);
            frame.map(CanFrame).ok()
        }

        fn is_extended(&self) -> bool {
            self.0.is_extended()
        }
        fn is_remote_frame(&self) -> bool {
            self.0.is_rtr()
        }

        fn dlc(&self) -> usize {
            self.0.data().len()
        }
        fn data(&self) -> &[u8] {
            self.0.data()
        }
        fn id(&self) -> Id {
            if self.is_extended() {
                let extended_id =
                    ExtendedId::new(self.0.id() & EFF_MASK).expect("Id exceeds max extend id");
                Id::Extended(extended_id)
            } else {
                let standard_id: u16 = self
                    .0
                    .id()
                    .try_into()
                    .expect("Id exceeds max standard id u16");
                let standard_id =
                    StandardId::new(standard_id & SFF_MASK).expect("Id exceeds max standard id");
                Id::Standard(standard_id)
            }
        }
    }

    // TODO figure out how to make sure socket is in non blocking mode
    // impl embedded_hal::can::nb::Can for CanSocket {
    //     type Frame = CanFrame;
    //     type Error = SocketcanError;
    //     fn transmit(
    //         &mut self,
    //         frame: &Self::Frame,
    //     ) -> Result<Option<Self::Frame>, nb::Error<Self::Error>> {
    //         self.write_frame(frame).map(|_| None).map_err(|io_err| {
    //             if io_err.kind() == io::ErrorKind::WouldBlock {
    //                 nb::Error::WouldBlock
    //             } else {
    //                 nb::Error::Other(SocketcanError::Io(io_err))
    //             }
    //         })
    //     }
    //     fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
    //         self.read_frame().map_err(|io_err| {
    //             if io_err.kind() == io::ErrorKind::WouldBlock {
    //                 nb::Error::WouldBlock
    //             } else {
    //                 nb::Error::Other(SocketcanError::Io(io_err))
    //             }
    //         })
    //     }
    // }

    /// Error type wrapping [io::Error](io::Error) to implement [embedded_hal::can::Error]
    #[derive(Debug)]
    pub struct CanError {
        pub err: io::Error,
    }
}
