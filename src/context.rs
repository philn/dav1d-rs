use ffi;
use std::ffi::c_void;
use std::{mem, ptr, rc};

use failure::Error;

#[derive(Debug, Fail)]
pub enum Dav1dError {
    #[fail(display = "decode error: {}", name)]
    DecodeError { name: String },
}
#[derive(Debug)]
pub struct Context {
    dec: *mut ffi::Dav1dContext,
}

#[derive(Debug)]
pub struct Picture {
    pic: rc::Rc<ffi::Dav1dPicture>,
}

impl Context {
    pub fn new() -> Self {
        unsafe {
            let mut settings = mem::uninitialized();
            let mut context: Context = mem::uninitialized();

            ffi::dav1d_default_settings(&mut settings);
            let ret = ffi::dav1d_open(&mut context.dec, &settings);

            if ret != 0 {
                panic!("Cannot instantiate the decoder {}", ret);
            }

            context
        }
    }

    pub fn close(&mut self) {
        unsafe {
            ffi::dav1d_close(&mut self.dec);
        }
    }

    pub fn flush(&self) {
        unsafe {
            ffi::dav1d_flush(self.dec);
        }
    }

    pub fn decode(&self, data: &[u8]) -> Result<Vec<Picture>, Dav1dError> {
        unsafe {
            let mut data_wrapper: ffi::Dav1dData = mem::zeroed();
            // println!(">> {}", data.len());
            let ptr = ffi::dav1d_data_create(&mut data_wrapper, data.len());
            ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
            let ret = ffi::dav1d_send_data(self.dec, &mut data_wrapper);
            // println!("Decode result {}", ret);
            if ret != 0 {
                return Err(Dav1dError::DecodeError {
                    name: "Data can't be consumed".to_string(),
                });
            }
            let mut pictures: Vec<Picture> = vec![];
            loop {
                let mut dpic: ffi::Dav1dPicture = mem::zeroed();
                let mut pic: Picture = mem::uninitialized();
                let ret = ffi::dav1d_get_picture(self.dec, &mut dpic);
                pic.pic = rc::Rc::new(dpic);
                if ret == -(ffi::EAGAIN as i32) {
                    // return Err(Dav1dError::DecodeError {
                    //     name: "Not enough data".to_string(),
                    // });
                    //panic!("foo");
                    break;
                } else if ret != 0 {
                    //panic!("bar");
                    return Err(Dav1dError::DecodeError {
                        name: "Invalid data".to_string(),
                    });
                }
                pictures.push(pic);
            }
            Ok(pictures)
        }
    }
}

pub enum PixelLayout {
    I400,
    I420,
    I422,
    I444,
    Unknown,
}

impl Picture {
    pub fn stride(&self, component: usize) -> i32 {
        (*self.pic).stride[component] as i32
    }

    pub fn plane_data(&self, component: usize) -> *mut c_void {
        (*self.pic).data[component]
    }

    pub fn bit_depth(&self) -> usize {
        (*self.pic).p.bpc as usize
    }

    pub fn width(&self) -> u32 {
        (*self.pic).p.w as u32
    }

    pub fn height(&self) -> u32 {
        (*self.pic).p.h as u32
    }

    pub fn pixel_layout(&self) -> PixelLayout {
        match (*self.pic).p.layout {
            ffi::Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I400 => PixelLayout::I400,
            ffi::Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I420 => PixelLayout::I420,
            ffi::Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I422 => PixelLayout::I422,
            ffi::Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I444 => PixelLayout::I444,
            _ => PixelLayout::Unknown,
        }
    }
}

impl Drop for Picture {
    fn drop(&mut self) {
        unsafe {
            ffi::dav1d_picture_unref(rc::Rc::get_mut(&mut self.pic).unwrap());
        }
    }
}

unsafe impl Send for Context {}
