function on_init() {}

register_plugin = function (importObject) {
    importObject.env.ws_connect = ws_connect;
    importObject.env.ws_is_connected = ws_is_connected;
    importObject.env.ws_send = ws_send;
    importObject.env.ws_try_recv = ws_try_recv;
}

miniquad_add_plugin({
    register_plugin,
    on_init,
    version: "0.1.1",
    name: "quad_net"
});

var quad_socket;
var connected = 0;
var received_buffer = [];

var global_error;

function ws_is_connected() {
    return connected;
}

function ws_connect(addr) {
    try {
        quad_socket = new WebSocket(consume_js_object(addr));
        quad_socket.binaryType = 'arraybuffer';
        quad_socket.onopen = function () {
            connected = 1;
        };

        quad_socket.onmessage = function (msg) {
            if (typeof msg.data == "string") {
                received_buffer.push({
                    ok: {
                        "text": 1,
                        "data": msg.data
                    }
                });
            } else {
                var buffer = new Uint8Array(msg.data);
                received_buffer.push({
                    ok: {
                        "text": 0,
                        "data": buffer
                    }
                });
            }
        }
        quad_socket.onerror = function (msg) {
            received_buffer.push({
                err: msg.toString(),
            });
        }
        // nil for good news
        return -1;
    } catch (e) {
        // oh no
        return js_object(e.toString());
    }
};

function ws_send(data) {
    var array = consume_js_object(data);
    // here should be a nice typecheck on array.is_string or whatever
    if (array.buffer != undefined) {
        quad_socket.send(array.buffer);
    } else {
        quad_socket.send(array);
    }
};

function ws_try_recv() {
    if (received_buffer.length != 0) {
        return js_object(received_buffer.shift())
    }
    return -1;
}