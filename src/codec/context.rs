use std::ops::Deref;
use std::ptr;

use ffi::*;
use ::media;
use ::{Error, Codec, Dictionary};
use super::Id;
use super::decoder::Decoder;
use super::encoder::Encoder;

pub struct Context {
	pub ptr: *mut AVCodecContext,

	_own: bool,
}

impl Context {
	pub fn new() -> Self {
		unsafe {
			Context { ptr: avcodec_alloc_context3(ptr::null()), _own: true }
		}
	}

	pub fn wrap(ptr: *mut AVCodecContext) -> Self {
		Context { ptr: ptr, _own: false }
	}

	pub fn open(self, codec: &Codec) -> Result<Opened, Error> {
		unsafe {
			match avcodec_open2(self.ptr, codec.ptr, ptr::null_mut()) {
				0 => Ok(Opened(self)),
				e => Err(Error::new(e))
			}
		}
	}

	pub fn open_with(self, codec: &Codec, mut options: Dictionary) -> Result<Opened, Error> {
		unsafe {
			match avcodec_open2(self.ptr, codec.ptr, &mut options.ptr) {
				0 => Ok(Opened(self)),
				e => Err(Error::new(e))
			}
		}
	}

	pub fn decoder(&self) -> Result<Decoder, Error> {
		if let Some(ref codec) = super::decoder::find(self.id()) {
			self.clone().open(codec).and_then(|c| c.decoder())
		}
		else {
			Err(Error::from(AVERROR_DECODER_NOT_FOUND))
		}
	}

	pub fn encoder(&self) -> Result<Encoder, Error> {
		if let Some(ref codec) = super::encoder::find(self.id()) {
			self.clone().open(codec).and_then(|c| c.encoder())
		}
		else {
			Err(Error::from(AVERROR_ENCODER_NOT_FOUND))
		}
	}

	pub fn codec(&self) -> Option<Codec> {
		unsafe {
			if (*self.ptr).codec == ptr::null() {
				None
			}
			else {
				Some(Codec::wrap((*self.ptr).codec as *mut _))
			}
		}
	}

	pub fn medium(&self) -> media::Type {
		unsafe {
			media::Type::from((*self.ptr).codec_type)
		}
	}

	pub fn id(&self) -> Id {
		unsafe {
			Id::from((*self.ptr).codec_id)
		}
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		if self._own {
			unsafe {
				avcodec_free_context(&mut self.ptr);
			}
		}
	}
}

impl Clone for Context {
	fn clone(&self) -> Self {
		let mut ctx = Context::new();
		ctx.clone_from(self);

		ctx
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			avcodec_copy_context(self.ptr, source.ptr);
		}
	}
}

pub struct Opened(pub Context);

impl Opened {
	pub fn decoder(self) -> Result<Decoder, Error> {
		let mut valid = false;

		if let Some(codec) = self.codec() {
			valid = codec.is_decoder();
		}

		if valid {
			Ok(Decoder(self))
		}
		else {
			Err(Error::from(AVERROR_INVALIDDATA))
		}
	}

	pub fn encoder(self) -> Result<Encoder, Error> {
		let mut valid = false;

		if let Some(codec) = self.codec() {
			valid = codec.is_encoder();
		}

		if valid {
			Ok(Encoder(self))
		}
		else {
			Err(Error::from(AVERROR_INVALIDDATA))
		}
	}
}

impl Drop for Opened {
	fn drop(&mut self) {
		unsafe {
			avcodec_close(self.0.ptr);
		}
	}
}

impl Deref for Opened {
	type Target = Context;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}