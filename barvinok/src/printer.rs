use std::fmt::Debug;

use barvinok_sys::isl_printer_flush;

struct CookieFile<'a, W: std::fmt::Write> {
    reference: &'a mut W,
    file: *mut barvinok_sys::FILE,
}
//        FILE *fopencookie(void *restrict cookie, const char *restrict mode,
//                          cookie_io_functions_t io_funcs);

#[repr(C)]
struct CookieFunctions {
    read:
        Option<unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_char, usize) -> isize>,
    write: Option<
        unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_char, usize) -> isize,
    >,
    seek: Option<unsafe extern "C" fn(*mut std::ffi::c_void, i64, i32) -> i64>,
    close: Option<unsafe extern "C" fn(*mut std::ffi::c_void) -> i32>,
}

unsafe extern "C" {
    fn fopencookie(
        cookie: *mut std::ffi::c_void,
        mode: *const std::ffi::c_char,
        io_funcs: CookieFunctions,
    ) -> *mut barvinok_sys::FILE;
    fn fclose(fp: *mut barvinok_sys::FILE) -> i32;
}

impl<'a, W: std::fmt::Write> CookieFile<'a, W> {
    fn new(reference: &'a mut W) -> Self {
        unsafe extern "C" fn write<W: std::fmt::Write>(
            cookie: *mut std::ffi::c_void,
            buf: *const std::ffi::c_char,
            size: usize,
        ) -> isize {
            let reference = cookie as *mut W;
            let slice = unsafe { std::slice::from_raw_parts(buf as *const u8, size) };
            let str = unsafe { std::str::from_utf8_unchecked(slice) };
            unsafe {
                match (*reference).write_str(str) {
                    Ok(_) => size as isize,
                    Err(_) => -1,
                }
            }
        }
        let cookie = CookieFunctions {
            read: None,
            write: Some(write::<W>),
            seek: None,
            close: None,
        };
        let file = unsafe {
            fopencookie(
                reference as *mut _ as *mut std::ffi::c_void,
                c"w".as_ptr(),
                cookie,
            )
        };
        Self { reference, file }
    }
}

impl<'a, W: std::fmt::Write> Drop for CookieFile<'a, W> {
    fn drop(&mut self) {
        unsafe { fclose(self.file) };
    }
}

pub trait ISLPrint<'a> {
    type Handle;

    fn context(&self) -> crate::ContextRef<'a>;
    fn handle(&self) -> *mut Self::Handle;
    unsafe fn isl_printer_print(
        printer: *mut barvinok_sys::isl_printer,
        handle: *mut Self::Handle,
    ) -> *mut barvinok_sys::isl_printer;
}

pub(crate) struct FmtWrapper<'a, 'b, T: ISLPrint<'b>>(&'a T, std::marker::PhantomData<&'b ()>);

impl<'a, 'b, T> FmtWrapper<'a, 'b, T>
where
    T: ISLPrint<'b>,
{
    pub fn new(handle: &'a T) -> Self {
        Self(handle, std::marker::PhantomData)
    }
}

impl<'b, T> std::fmt::Debug for FmtWrapper<'_, 'b, T>
where
    T: ISLPrint<'b>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cookie = CookieFile::new(f);
        let ctx = self.0.context();
        let handle = self.0.handle();
        let printer = unsafe { barvinok_sys::isl_printer_to_file(ctx.0.as_ptr(), cookie.file) };
        let printer = unsafe { T::isl_printer_print(printer, handle) };
        let printer = unsafe { isl_printer_flush(printer) };
        unsafe { barvinok_sys::isl_printer_free(printer) };
        Ok(())
    }
}

impl<'b, T> std::fmt::Display for FmtWrapper<'_, 'b, T>
where
    T: ISLPrint<'b>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
