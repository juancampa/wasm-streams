use std::marker::PhantomData;

use futures::Sink;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub use into_sink::IntoSink;

use crate::writable::into_underlying_sink::IntoUnderlyingSink;

mod into_sink;
mod into_underlying_sink;
pub mod sys;

#[derive(Debug)]
pub struct WritableStream {
    raw: sys::WritableStream,
}

impl WritableStream {
    #[inline]
    pub fn from_raw(raw: sys::WritableStream) -> Self {
        Self { raw }
    }

    #[inline]
    pub fn as_raw(&self) -> &sys::WritableStream {
        &self.raw
    }

    #[inline]
    pub fn into_raw(self) -> sys::WritableStream {
        self.raw
    }

    pub fn is_locked(&self) -> bool {
        self.raw.is_locked()
    }

    pub async fn abort(&mut self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.raw.abort()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn abort_with_reason(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.raw.abort_with_reason(reason)).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub fn get_writer(&mut self) -> Result<WritableStreamDefaultWriter, JsValue> {
        Ok(WritableStreamDefaultWriter {
            raw: Some(self.raw.get_writer()?),
            _stream: PhantomData,
        })
    }

    pub fn into_sink(self) -> Result<IntoSink<'static>, (Self, JsValue)> {
        let raw_writer = match self.raw.get_writer() {
            Ok(raw_writer) => raw_writer,
            Err(err) => return Err((self, err)),
        };
        let writer = WritableStreamDefaultWriter {
            raw: Some(raw_writer),
            _stream: PhantomData,
        };
        Ok(writer.into_sink())
    }
}

impl<Si> From<Si> for WritableStream
where
    Si: Sink<JsValue, Error = JsValue> + 'static,
{
    fn from(sink: Si) -> Self {
        let sink = IntoUnderlyingSink::new(Box::new(sink));
        // Use the default queuing strategy (with a HWM of 1 chunk).
        // We shouldn't set HWM to 0, since that would break piping to the writable stream.
        let raw = sys::WritableStream::new_with_sink(sink);
        WritableStream { raw }
    }
}

#[derive(Debug)]
pub struct WritableStreamDefaultWriter<'stream> {
    raw: Option<sys::WritableStreamDefaultWriter>,
    _stream: PhantomData<&'stream mut WritableStream>,
}

impl<'stream> WritableStreamDefaultWriter<'stream> {
    #[inline]
    pub fn as_raw(&self) -> &sys::WritableStreamDefaultWriter {
        self.raw.as_ref().unwrap_throw()
    }

    pub async fn closed(&self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().closed()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub fn desired_size(&self) -> Option<f64> {
        self.as_raw().desired_size()
    }

    pub async fn ready(&self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().ready()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn abort(&mut self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().abort()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn abort_with_reason(&mut self, reason: &JsValue) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().abort_with_reason(reason)).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn write(&mut self, chunk: JsValue) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().write(chunk)).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), JsValue> {
        let js_value = JsFuture::from(self.as_raw().close()).await?;
        debug_assert!(js_value.is_undefined());
        Ok(())
    }

    pub fn release_lock(&mut self) -> Result<(), JsValue> {
        if let Some(raw) = self.raw.as_ref() {
            raw.release_lock()?;
            self.raw.take();
        }
        Ok(())
    }

    pub fn into_sink(self) -> IntoSink<'stream> {
        IntoSink::new(self)
    }
}

impl Drop for WritableStreamDefaultWriter<'_> {
    fn drop(&mut self) {
        // TODO Error handling?
        self.release_lock().unwrap_throw();
    }
}
