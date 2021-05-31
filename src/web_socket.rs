//! Websocket client. Works through native websockets on web and through ws-rs on the desktop.

#[cfg(target_arch = "wasm32")]
pub(crate) mod js_web_socket {
    use std::{io::ErrorKind, net::ToSocketAddrs};

    use sapp_jsutils::JsObject;

    use crate::error::Error;

    pub struct WebSocket;

    extern "C" {
        fn ws_connect(addr: JsObject) -> JsObject;
        fn ws_send(buffer: JsObject);
        fn ws_try_recv() -> JsObject;
        fn ws_is_connected() -> i32;
    }

    impl WebSocket {
        pub fn send_text(&self, text: &str) {
            unsafe { ws_send(JsObject::string(text)) };
        }

        pub fn send_bytes(&self, data: &[u8]) {
            unsafe { ws_send(JsObject::buffer(data)) };
        }

        /// Returns `None` if there was nothing to receive.
        ///
        /// Returns `Ok` with the data if any, or an error.
        pub fn try_recv(&mut self) -> Option<Result<Vec<u8>, Error>> {
            let data = unsafe { ws_try_recv() };
            if data.is_nil() == false {
                // returns: {ok: {}} | {err: string}
                // this function name is backwards
                if !data.have_field("err") {
                    // oh no
                    let mut out = String::new();
                    data.field("err").to_string(&mut out);
                    return Some(Err(Error::Misc(out)));
                } else {
                    let data = data.field("ok");
                    let is_text = data.field_u32("text") == 1;
                    let mut buf = vec![];
                    if is_text {
                        let mut s = String::new();
                        data.field("data").to_string(&mut s);
                        buf = s.into_bytes();
                    } else {
                        data.field("data").to_byte_buffer(&mut buf);
                    }
                    return Some(Ok(buf));
                }
            }
            None
        }

        pub fn connected(&self) -> bool {
            unsafe { ws_is_connected() == 1 }
        }

        pub fn connect<A: ToSocketAddrs + std::fmt::Display>(addr: A) -> Result<WebSocket, Error> {
            let conn = unsafe { ws_connect(JsObject::string(&format!("{}", addr))) };
            if !conn.is_nil() {
                // uh oh
                let mut buf = String::new();
                conn.to_string(&mut buf);
                Err(Error::IOError(std::io::Error::new(ErrorKind::Other, buf)))
            } else {
                Ok(WebSocket)
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod pc_web_socket {
    use std::net::ToSocketAddrs;
    use std::sync::{mpsc, Mutex};

    use crate::error::Error;

    pub struct WebSocket {
        sender: ws::Sender,
        rx: Mutex<mpsc::Receiver<Event>>,
    }

    enum Event {
        Connect(ws::Sender),
        Message(Vec<u8>),
    }

    struct Client {
        out: ws::Sender,
        thread_out: mpsc::Sender<Event>,
    }

    impl ws::Handler for Client {
        fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
            self.thread_out
                .send(Event::Connect(self.out.clone()))
                .unwrap();
            Ok(())
        }

        fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
            self.thread_out
                .send(Event::Message(msg.into_data()))
                .unwrap();
            Ok(())
        }

        fn on_close(&mut self, code: ws::CloseCode, _reason: &str) {
            println!("closed {:?}", code);
        }

        fn on_error(&mut self, error: ws::Error) {
            println!("{:?}", error);
        }
    }

    impl WebSocket {
        pub fn connect<A: ToSocketAddrs + std::fmt::Display>(addr: A) -> Result<WebSocket, Error> {
            let (tx, rx) = mpsc::channel();
            let ws_addr = format!("{}", addr);
            std::thread::spawn(move || {
                ws::connect(ws_addr, |out| Client {
                    out,
                    thread_out: tx.clone(),
                })
                .unwrap()
            });

            match rx.recv() {
                Ok(Event::Connect(sender)) => Ok(WebSocket {
                    sender,
                    rx: Mutex::new(rx),
                }),
                _ => panic!("Failed to connect websocket"),
            }
        }

        pub fn connected(&self) -> bool {
            true
        }

        pub fn try_recv(&mut self) -> Option<Vec<u8>> {
            self.rx
                .lock()
                .unwrap()
                .try_recv()
                .ok()
                .map(|event| match event {
                    Event::Message(msg) => msg,
                    _ => panic!(),
                })
        }

        pub fn send_text(&self, text: &str) {
            self.sender.send(ws::Message::text(text)).unwrap();
        }

        pub fn send_bytes(&self, data: &[u8]) {
            self.sender
                .send(ws::Message::Binary(data.to_vec()))
                .unwrap();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use js_web_socket::WebSocket;

#[cfg(not(target_arch = "wasm32"))]
pub use pc_web_socket::WebSocket;
