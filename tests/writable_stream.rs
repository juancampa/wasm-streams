extern crate wasm_bindgen_test;

use futures::channel::*;
use futures::{SinkExt, StreamExt};
use pin_utils::pin_mut;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use wasm_streams::writable::*;

#[wasm_bindgen(module = "/tests/writable_stream.js")]
extern "C" {
    fn new_noop_writable_stream() -> sys::WritableStream;
    fn new_logging_writable_stream() -> sys::WritableStream;
}

#[wasm_bindgen_test]
async fn test_writable_stream_new() {
    let mut writable = WritableStream::from_raw(new_noop_writable_stream());
    assert!(!writable.is_locked());

    let mut writer = writable.get_writer().unwrap();
    assert_eq!(writer.write(JsValue::from("Hello")).await.unwrap(), ());
    assert_eq!(writer.write(JsValue::from("world!")).await.unwrap(), ());
    assert_eq!(writer.close().await.unwrap(), ());
    writer.closed().await.unwrap();
}

#[wasm_bindgen_test]
async fn test_writable_stream_into_sink() {
    let mut writable = WritableStream::from_raw(new_logging_writable_stream());
    assert!(!writable.is_locked());

    let writer = writable.get_writer().unwrap();
    let sink = writer.into_sink();
    pin_mut!(sink);
    assert_eq!(sink.send(JsValue::from("Hello")).await, Ok(()));
    assert_eq!(sink.send(JsValue::from("world!")).await, Ok(()));
    assert_eq!(sink.close().await, Ok(()));
}

#[wasm_bindgen_test]
async fn test_writable_stream_from_sink() {
    let (sink, stream) = mpsc::unbounded::<JsValue>();
    let sink = sink.sink_map_err(|_| JsValue::from_str("cannot happen"));
    let mut writable = WritableStream::from(sink);

    let mut writer = writable.get_writer().unwrap();
    assert_eq!(writer.write(JsValue::from("Hello")).await.unwrap(), ());
    assert_eq!(writer.write(JsValue::from("world!")).await.unwrap(), ());
    assert_eq!(writer.close().await.unwrap(), ());
    writer.closed().await.unwrap();

    let output = stream.collect::<Vec<_>>().await;
    assert_eq!(
        output,
        vec![JsValue::from("Hello"), JsValue::from("world!")]
    );
}
