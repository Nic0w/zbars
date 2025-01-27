use crate::{
    as_char_ptr,
    ffi,
    format::Format,
    image::ZBarImage,
    symbol_set::ZBarSymbolSet,
    ZBarConfig,
    ZBarErrorType,
    ZBarResult,
    ZBarSymbolType,
};
use std::ptr;

pub struct ZBarProcessor {
    processor: *mut ffi::zbar_processor_s,
}
impl ZBarProcessor {
    pub fn new(threaded: bool) -> Self {
        let mut processor = ZBarProcessor {
            processor: unsafe { ffi::zbar_processor_create(threaded as i32) },
        };
        processor
            .set_config(ZBarSymbolType::ZBAR_NONE, ZBarConfig::ZBAR_CFG_ENABLE, 0)
            // save to unwrap here
            .unwrap();
        processor
    }
    pub fn builder() -> ZBarProcessorBuilder {
        ZBarProcessorBuilder::new()
    }

    //Tested
    pub fn init(&self, video_device: impl AsRef<str>, enable_display: bool) -> ZBarResult<()> {
        match unsafe {
            ffi::zbar_processor_init(
                self.processor,
                as_char_ptr(video_device),
                enable_display as i32,
            )
        } {
            0 => Ok(()),
            e => Err(ZBarErrorType::Simple(e)),
        }
    }
    //Tested
    pub fn request_size(&self, width: u32, height: u32) -> ZBarResult<()> {
        match unsafe { ffi::zbar_processor_request_size(self.processor, width, height) } {
            0 => Ok(()),
            e => Err(ZBarErrorType::Simple(e)),
        }
    }
    //Tested
    pub fn request_interface(&self, version: i32) -> ZBarResult<()> {
        match unsafe { ffi::zbar_processor_request_interface(self.processor, version) } {
            0 => Ok(()),
            e => Err(ZBarErrorType::Simple(e)),
        }
    }
    //Tested
    pub fn request_iomode(&self, iomode: i32) -> ZBarResult<()> {
        match unsafe { ffi::zbar_processor_request_iomode(self.processor, iomode) } {
            0 => Ok(()),
            e => Err(ZBarErrorType::Simple(e)),
        }
    }
    pub fn force_format(&self, input_format: Format, output_format: Format) -> ZBarResult<()> {
        match unsafe {
            ffi::zbar_processor_force_format(
                self.processor,
                input_format.value().into(),
                output_format.value().into(),
            )
        } {
            0 => Ok(()),
            e => Err(ZBarErrorType::Simple(e)),
        }
    }

    pub fn set_config(
        &mut self,
        symbol_type: ZBarSymbolType,
        config: ZBarConfig,
        value: i32,
    ) -> ZBarResult<()> {
        match unsafe { ffi::zbar_processor_set_config(self.processor, symbol_type, config, value) }
        {
            0 => Ok(()),
            e => Err(e.into()),
        }
    }

    pub fn is_visible(&self) -> ZBarResult<bool> {
        match unsafe { ffi::zbar_processor_is_visible(self.processor) } {
            0 => Ok(false),
            1 => Ok(true),
            e => Err(ZBarErrorType::Simple(e)),
        }
    }
    pub fn set_visible(&self, visible: bool) -> ZBarResult<bool> {
        match unsafe { ffi::zbar_processor_set_visible(self.processor, visible as i32) } {
            0 => Ok(false),
            1 => Ok(true),
            e => Err(ZBarErrorType::Simple(e)),
        }
    }
    pub fn set_active(&self, active: bool) -> ZBarResult<bool> {
        match unsafe { ffi::zbar_processor_set_active(self.processor, active as i32) } {
            0 => Ok(false),
            1 => Ok(true),
            e => Err(ZBarErrorType::Simple(e)),
        }
    }
    pub fn get_results(&self) -> Option<ZBarSymbolSet> {
        ZBarSymbolSet::from_raw(
            unsafe { ffi::zbar_processor_get_results(self.processor) },
            ptr::null_mut(),
        )
    }

    // Tested
    pub fn user_wait(&self, timeout: i32) -> ZBarResult<i32> {
        match unsafe { ffi::zbar_processor_user_wait(self.processor, timeout) } {
            -1 => Err(ZBarErrorType::Simple(-1)),
            o => Ok(o),
        }
    }

    // Tested
    pub fn process_one(&self, timeout: i32) -> ZBarResult<Option<ZBarSymbolSet>> {
        match unsafe { ffi::zbar_process_one(self.processor, timeout) } {
            -1 => Err(ZBarErrorType::Simple(-1)),
            0 => Ok(None),
            _ => Ok(self.get_results()),
        }
    }

    // Tested
    pub fn process_image<T>(&self, image: &ZBarImage<T>) -> ZBarResult<ZBarSymbolSet> {
        match unsafe { ffi::zbar_process_image(self.processor, image.image()) } {
            -1 => Err(ZBarErrorType::Simple(-1)),
            _ => Ok(image.symbols().unwrap()), // symbols can be unwrapped because image is surely scanned
        }
    }
}
#[cfg(feature = "zbar_fork")]
impl ZBarProcessor {
    /// Set V4L2 Controls.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use zbars::prelude::*;
    ///
    /// let processor = ZBarProcessor::builder().build().unwrap();
    /// processor.init("/dev/video0", false).unwrap();
    /// processor.set_control("brightness", 75).unwrap();
    /// processor.set_control("contrast", 50).unwrap();
    /// ```
    pub fn set_control(&self, control_name: impl AsRef<str>, value: i32) -> ZBarResult<()> {
        match unsafe {
            ffi::zbar_processor_set_control(self.processor, as_char_ptr(control_name), value)
        } {
            0 => Ok(()),
            e => Err(ZBarErrorType::Simple(e)),
        }
    }

    /// Get V4L2 Controls.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use zbars::prelude::*;
    ///
    /// let processor = ZBarProcessor::builder().build().unwrap();
    /// processor.init("/dev/video0", false).unwrap();
    /// println!("brightness: {}", processor.control("brightness").unwrap());
    /// println!("contrast: {}", processor.control("contrast").unwrap());
    /// ```
    pub fn control(&self, control_name: impl AsRef<str>) -> ZBarResult<i32> {
        let mut value = 0;
        match unsafe {
            ffi::zbar_processor_get_control(
                self.processor,
                as_char_ptr(control_name),
                &mut value as *mut i32,
            )
        } {
            0 => Ok(value),
            e => Err(ZBarErrorType::Simple(e)),
        }
    }
}
unsafe impl Send for ZBarProcessor {}
unsafe impl Sync for ZBarProcessor {}

impl Drop for ZBarProcessor {
    fn drop(&mut self) {
        unsafe { ffi::zbar_processor_destroy(self.processor) }
    }
}

#[derive(Default)]
pub struct ZBarProcessorBuilder {
    threaded: bool,
    size: Option<(u32, u32)>,
    interface_version: Option<i32>,
    iomode: Option<i32>,
    format: Option<(Format, Format)>,
    config: Vec<(ZBarSymbolType, ZBarConfig, i32)>,
}
impl ZBarProcessorBuilder {
    pub fn new() -> Self {
        Self {
            threaded: false,
            size: None,
            interface_version: None,
            iomode: None,
            format: None,
            config: vec![],
        }
    }
    pub fn threaded(&mut self, threaded: bool) -> &mut Self {
        self.threaded = threaded;
        self
    }
    pub fn with_size(&mut self, size: Option<(u32, u32)>) -> &mut Self {
        self.size = size;
        self
    }
    pub fn with_interface_version(&mut self, interface_version: Option<i32>) -> &mut Self {
        self.interface_version = interface_version;
        self
    }
    pub fn with_iomode(&mut self, iomode: Option<i32>) -> &mut Self {
        self.iomode = iomode;
        self
    }
    pub fn with_format(&mut self, format: Option<(Format, Format)>) -> &mut Self {
        self.format = format;
        self
    }
    pub fn with_config(
        &mut self,
        symbol_type: ZBarSymbolType,
        config: ZBarConfig,
        value: i32,
    ) -> &mut Self {
        self.config.push((symbol_type, config, value));
        self
    }
    pub fn build(&self) -> ZBarResult<ZBarProcessor> {
        let mut processor = ZBarProcessor::new(self.threaded);
        if let Some(size) = self.size {
            processor.request_size(size.0, size.1)?;
        }
        if let Some(interface_version) = self.interface_version {
            processor.request_interface(interface_version)?;
        }
        if let Some(iomode) = self.iomode {
            processor.request_iomode(iomode)?;
        }
        if let Some(ref format) = self.format {
            processor.force_format(format.0, format.1)?;
        }
        self.config
            .iter()
            .try_for_each(|v| processor.set_config(v.0, v.1, v.2))
            .map(|_| processor)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_wrong_video_device() {
        let processor = ZBarProcessor::builder().threaded(true).build().unwrap();

        assert!(processor.init("nothing", true).is_err())
    }

    #[test]
    #[cfg(feature = "from_image")]
    fn test_process_image() {
        let image = ZBarImage::from_path("test/qr_hello-world.png").unwrap();

        let processor = ZBarProcessor::builder()
            .threaded(true)
            .with_config(ZBarSymbolType::ZBAR_QRCODE, ZBarConfig::ZBAR_CFG_ENABLE, 1)
            .with_config(ZBarSymbolType::ZBAR_CODE128, ZBarConfig::ZBAR_CFG_ENABLE, 1)
            .build()
            .unwrap();

        processor.process_image(&image).unwrap();

        let symbol = image.first_symbol().unwrap();

        assert_eq!(symbol.symbol_type(), ZBarSymbolType::ZBAR_QRCODE);
        assert_eq!(symbol.data(), "Hello World");
        assert_eq!(symbol.next().is_none(), true);
    }

    #[test]
    #[cfg(feature = "zbar_fork")]
    fn test_control_get_set() {
        test_control();
        test_set_control();
    }

    #[cfg(feature = "zbar_fork")]
    fn test_set_control() {
        let processor = ZBarProcessor::builder()
            .threaded(true)
            .with_config(ZBarSymbolType::ZBAR_QRCODE, ZBarConfig::ZBAR_CFG_ENABLE, 1)
            .with_config(ZBarSymbolType::ZBAR_CODE128, ZBarConfig::ZBAR_CFG_ENABLE, 1)
            .build()
            .unwrap();

        processor.init("/dev/video0", false).unwrap();
        assert!(processor.set_control("brightness", 100).is_ok());
        assert!(processor.set_control("contrast", 100).is_ok());
    }

    #[cfg(feature = "zbar_fork")]
    fn test_control() {
        let processor = ZBarProcessor::builder()
            .threaded(true)
            .with_config(ZBarSymbolType::ZBAR_QRCODE, ZBarConfig::ZBAR_CFG_ENABLE, 1)
            .with_config(ZBarSymbolType::ZBAR_CODE128, ZBarConfig::ZBAR_CFG_ENABLE, 1)
            .build()
            .unwrap();

        processor.init("/dev/video0", false).unwrap();
        assert!(processor.control("brightness").is_ok());
        assert!(processor.control("contrast").is_ok());
    }
}
