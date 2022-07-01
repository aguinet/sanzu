use anyhow::{Context, Result};
use ffmpeg::{AVPixelFormat, av_image_fill_pointers};
use ffmpeg_sys_next as ffmpeg;
use std::ffi::{CStr, CString};

pub fn averror(msg: &str, num: i32) -> anyhow::Error {
    let mut buf = vec![0u8; 200];
    if unsafe { ffmpeg::av_strerror(num, buf.as_mut_ptr() as *mut _, buf.len()) } == 0 {
        if let Ok(msg_err) =
            CStr::from_bytes_with_nul(&buf[..]).map(|s| s.to_string_lossy().into_owned())
        {
            return anyhow!("EncoderError {} {}", msg, msg_err);
        }
    }
    anyhow!("{} Undefined error {:?}", msg, num)
}

/// # Safety
/// obj must be a correct ffmpeg context
pub unsafe fn set_option(obj: *mut libc::c_void, name: &str, val: &str) -> Result<()> {
    let name_c = CString::new(name).context("Error in CString")?;
    let val_c = CString::new(val).context("Error in CString")?;
    let retval: i32 = ffmpeg::av_opt_set(
        obj,
        name_c.as_ptr(),
        val_c.as_ptr(),
        ffmpeg::AV_OPT_SEARCH_CHILDREN,
    );
    if retval != 0 {
        return Err(averror("set_option", retval));
    }
    Ok(())
}

/// Hold information on the FFmpg codec context
#[derive(Debug)]
pub struct AVCodecContext {
    /// Raw pointer on the AVCodecContext
    ptr: *mut ffmpeg::AVCodecContext,
}

impl AVCodecContext {
    pub fn new(codec: &AVCodec) -> Result<Self> {
        let ptr = unsafe { ffmpeg::avcodec_alloc_context3(codec.as_ptr()) };
        if ptr.is_null() {
            return Err(anyhow!("Error in avcodec_alloc_context3"));
        }
        Ok(AVCodecContext { ptr })
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffmpeg::AVCodecContext {
        self.ptr
    }
}

impl Drop for AVCodecContext {
    fn drop(&mut self) {
        unsafe {
            ffmpeg::avcodec_free_context(&mut self.ptr);
        }
    }
}

/// Hold information on the FFmpg codec context
#[derive(Debug)]
pub struct AVCodec {
    /// Raw pointer on the AVCodec
    ptr: *const ffmpeg::AVCodec,
}

impl AVCodec {
    pub fn new_encoder(name: &str) -> Result<Self> {
        let name_c = CString::new(name).context("Error in CString")?;
        let ptr = unsafe { ffmpeg::avcodec_find_encoder_by_name(name_c.as_ptr()) };
        if ptr.is_null() {
            Err(anyhow!("CodecNotFound: {}", name))
        } else {
            Ok(AVCodec { ptr })
        }
    }

    pub fn new_decoder(name: &str) -> Result<Self> {
        let name_c = CString::new(name).context("Error in CString")?;
        let ptr = unsafe { ffmpeg::avcodec_find_decoder_by_name(name_c.as_ptr()) };
        if ptr.is_null() {
            Err(anyhow!("CodecNotFound: {}", name))
        } else {
            Ok(AVCodec { ptr })
        }
    }

    pub fn id(&self) -> i32 {
        unsafe { *self.ptr }.id as i16 as i32
    }
    pub fn as_ptr(&self) -> *const ffmpeg::AVCodec {
        self.ptr
    }
}

/// Hold information on the FFmpg packet
#[derive(Debug)]
pub struct AVPacket {
    /// Raw pointer on the AVPacket
    ptr: *mut ffmpeg::AVPacket,
}

impl AVPacket {
    pub fn new() -> Result<Self> {
        let ptr = unsafe { ffmpeg::av_packet_alloc() };
        if ptr.is_null() {
            Err(anyhow!("Error in av_packet_alloc"))
        } else {
            Ok(AVPacket { ptr })
        }
    }

    pub fn as_mut_ptr(&self) -> *mut ffmpeg::AVPacket {
        self.ptr
    }
}

impl Drop for AVPacket {
    fn drop(&mut self) {
        unsafe {
            ffmpeg::av_packet_free(&mut self.ptr);
        }
    }
}

/// Hold information on the FFmpg frame
#[derive(Debug)]
pub struct AVFrame {
    /// Raw pointer on the AVFrame
    pub ptr: *mut ffmpeg::AVFrame,
}

impl AVFrame {
    pub fn new() -> Result<Self> {
        let ptr = unsafe { ffmpeg::av_frame_alloc() };
        if ptr.is_null() {
            Err(anyhow!("Error in av_frame_alloc"))
        } else {
            Ok(AVFrame { ptr })
        }
    }

    pub fn as_mut_ptr(&self) -> *mut ffmpeg::AVFrame {
        self.ptr
    }

    pub fn make_writable(&mut self) -> Result<()> {
        let retval: i32 = unsafe { ffmpeg::av_frame_make_writable(self.ptr) };
        if retval < 0 {
            return Err(averror("av_frame_make", retval));
        }
        Ok(())
    }

    pub fn rgb0_from_slice(&mut self, data: &[u8], bytes_per_line: i32) {
        unsafe {
            let frame_ptr = &mut *self.as_mut_ptr();
            frame_ptr.linesize[0] = bytes_per_line;
            frame_ptr.linesize[1] = 0;
            frame_ptr.linesize[2] = 0;
            frame_ptr.data[0] = data.as_ptr() as *mut u8;
            frame_ptr.data[1] = 0 as *mut u8;
            frame_ptr.data[2] = 0 as *mut u8;
            //av_image_fill_pointers(frame_ptr.data.as_mut_ptr(), AVPixelFormat::AV_PIX_FMT_RGB0, frame_ptr.height, data.as_ptr() as *mut u8, (*self.ptr).linesize.as_ptr());
        }
    }

    pub fn plane(&mut self, indx: usize, len: usize) -> &mut [u8] {
        unsafe {
            let data_ptr = (*self.ptr).data[indx] as *mut u8;
            std::slice::from_raw_parts_mut(data_ptr, len)
        }
    }
}

impl Drop for AVFrame {
    fn drop(&mut self) {
        unsafe {
            ffmpeg::av_frame_free(&mut self.ptr);
        }
    }
}

/// Hold information on the FFmpg parser
#[derive(Debug)]
pub struct AVParser {
    /// Raw pointer on the AVParser
    ptr: *mut ffmpeg::AVCodecParserContext,
}

impl AVParser {
    pub fn new(codec_id: i32) -> Result<Self> {
        let ptr = unsafe { ffmpeg::av_parser_init(codec_id) };
        if ptr.is_null() {
            Err(anyhow!("Error in av_parser_init"))
        } else {
            Ok(AVParser { ptr })
        }
    }

    pub fn as_mut_ptr(&self) -> *mut ffmpeg::AVCodecParserContext {
        self.ptr
    }
}

impl Drop for AVParser {
    fn drop(&mut self) {
        unsafe {
            ffmpeg::av_parser_close(self.ptr);
        }
    }
}
