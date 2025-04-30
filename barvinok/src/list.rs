use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::{ContextRef, nonnull_or_alloc_error, printer::ISLPrint};

#[allow(clippy::missing_safety_doc)]
pub trait ListRawAPI {
    type Handle;
    type ListHandle;

    unsafe fn get_handle(&self) -> *mut Self::Handle;
    unsafe fn from_raw_handle(handle: NonNull<Self::Handle>) -> Self;

    unsafe fn get_context(list: *mut Self::ListHandle) -> *mut barvinok_sys::isl_ctx;

    unsafe fn to_list(el: *mut Self::Handle) -> *mut Self::ListHandle;

    unsafe fn list_from_el(el: *mut Self::Handle) -> *mut Self::ListHandle;

    unsafe fn list_alloc(ctx: *mut barvinok_sys::isl_ctx, n: i32) -> *mut Self::ListHandle;

    unsafe fn list_copy(list: *mut Self::ListHandle) -> *mut Self::ListHandle;

    unsafe fn list_free(list: *mut Self::ListHandle) -> *mut Self::ListHandle;

    unsafe fn list_add(list: *mut Self::ListHandle, el: *mut Self::Handle)
    -> *mut Self::ListHandle;

    unsafe fn list_insert(
        list: *mut Self::ListHandle,
        pos: u32,
        el: *mut Self::Handle,
    ) -> *mut Self::ListHandle;

    unsafe fn list_drop(list: *mut Self::ListHandle, first: u32, n: u32) -> *mut Self::ListHandle;

    unsafe fn list_clear(list: *mut Self::ListHandle) -> *mut Self::ListHandle;

    unsafe fn list_swap(list: *mut Self::ListHandle, pos1: u32, pos2: u32)
    -> *mut Self::ListHandle;

    unsafe fn list_reverse(list: *mut Self::ListHandle) -> *mut Self::ListHandle;

    unsafe fn list_concat(
        list1: *mut Self::ListHandle,
        list2: *mut Self::ListHandle,
    ) -> *mut Self::ListHandle;

    unsafe fn list_size(list: *mut Self::ListHandle) -> isize;

    unsafe fn list_get_at(list: *mut Self::ListHandle, index: i32) -> *mut Self::Handle;

    unsafe fn list_set_at(
        list: *mut Self::ListHandle,
        index: i32,
        el: *mut Self::Handle,
    ) -> *mut Self::ListHandle;

    unsafe fn list_foreach(
        list: *mut Self::ListHandle,
        fn_ptr: unsafe extern "C" fn(*mut Self::Handle, *mut std::ffi::c_void) -> i32,
        user: *mut std::ffi::c_void,
    ) -> i32;

    unsafe fn list_every(
        list: *mut Self::ListHandle,
        test: unsafe extern "C" fn(*mut Self::Handle, *mut std::ffi::c_void) -> i32,
        user: *mut std::ffi::c_void,
    ) -> i32;

    unsafe fn list_map(
        list: *mut Self::ListHandle,
        fn_ptr: unsafe extern "C" fn(*mut Self::Handle, *mut std::ffi::c_void) -> *mut Self::Handle,
        user: *mut std::ffi::c_void,
    ) -> *mut Self::ListHandle;

    unsafe fn list_sort(
        list: *mut Self::ListHandle,
        cmp: unsafe extern "C" fn(
            *mut Self::Handle,
            *mut Self::Handle,
            *mut std::ffi::c_void,
        ) -> i32,
        user: *mut std::ffi::c_void,
    ) -> *mut Self::ListHandle;

    unsafe fn list_foreach_scc(
        list: *mut Self::ListHandle,
        follows: unsafe extern "C" fn(
            *mut Self::Handle,
            *mut Self::Handle,
            *mut std::ffi::c_void,
        ) -> i32,
        follows_user: *mut std::ffi::c_void,
        fn_ptr: unsafe extern "C" fn(*mut Self::ListHandle, *mut std::ffi::c_void) -> i32,
        fn_user: *mut std::ffi::c_void,
    ) -> i32;

    unsafe fn list_to_str(list: *mut Self::ListHandle) -> *mut std::os::raw::c_char;

    unsafe fn printer_print_list(
        p: *mut barvinok_sys::isl_printer,
        list: *mut Self::ListHandle,
    ) -> *mut barvinok_sys::isl_printer;

    unsafe fn list_dump(list: *mut Self::ListHandle);
}

macro_rules! impl_list_raw_api {
    ($t:ty, handle = $handle:ty, list_handle = $list_handle:ty, prefix = $prefix:ident, $get_handle:stmt, $from_raw_handle:stmt) => {
        paste::paste! {
            #[allow(unsafe_op_in_unsafe_fn)]
            impl ListRawAPI for $t {
                type Handle = $handle;
                type ListHandle = $list_handle;

                $get_handle

                $from_raw_handle

                unsafe fn get_context(list: *mut Self::ListHandle) -> *mut barvinok_sys::isl_ctx {
                    barvinok_sys::[<isl_ $prefix _list_get_ctx>](list)
                }

                unsafe fn to_list(el: *mut Self::Handle) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _to_list>](el)
                }

                unsafe fn list_from_el(el: *mut Self::Handle) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_from_ $prefix>](el)
                }

                unsafe fn list_alloc(ctx: *mut barvinok_sys::isl_ctx, n: i32) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_alloc>](ctx, n)
                }

                unsafe fn list_copy(list: *mut Self::ListHandle) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_copy>](list)
                }

                unsafe fn list_free(list: *mut Self::ListHandle) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_free>](list)
                }

                unsafe fn list_add(list: *mut Self::ListHandle, el: *mut Self::Handle) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_add>](list, el)
                }

                unsafe fn list_insert(list: *mut Self::ListHandle, pos: u32, el: *mut Self::Handle) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_insert>](list, pos, el)
                }

                unsafe fn list_drop(list: *mut Self::ListHandle, first: u32, n: u32) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_drop>](list, first, n)
                }

                unsafe fn list_clear(list: *mut Self::ListHandle) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_clear>](list)
                }

                unsafe fn list_swap(list: *mut Self::ListHandle, pos1: u32, pos2: u32) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_swap>](list, pos1, pos2)
                }

                unsafe fn list_reverse(list: *mut Self::ListHandle) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_reverse>](list)
                }

                unsafe fn list_concat(list1: *mut Self::ListHandle, list2: *mut Self::ListHandle) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_concat>](list1, list2)
                }

                unsafe fn list_size(list: *mut Self::ListHandle) -> isize {
                    barvinok_sys::[<isl_ $prefix _list_size>](list) as isize
                }

                unsafe fn list_get_at(list: *mut Self::ListHandle, index: i32) -> *mut Self::Handle {
                    barvinok_sys::[<isl_ $prefix _list_get_at>](list, index)
                }

                unsafe fn list_set_at(list: *mut Self::ListHandle, index: i32, el: *mut Self::Handle) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_set_at>](list, index, el)
                }

                unsafe fn list_foreach(
                    list: *mut Self::ListHandle,
                    fn_ptr: unsafe extern "C" fn(*mut Self::Handle, *mut std::ffi::c_void) -> i32,
                    user: *mut std::ffi::c_void,
                ) -> i32 {
                    barvinok_sys::[<isl_ $prefix _list_foreach>](list, Some(fn_ptr), user)
                }

                unsafe fn list_every(
                    list: *mut Self::ListHandle,
                    test: unsafe extern "C" fn(*mut Self::Handle, *mut std::ffi::c_void) -> i32,
                    user: *mut std::ffi::c_void,
                ) -> i32 {
                    barvinok_sys::[<isl_ $prefix _list_every>](list, Some(test), user)
                }

                unsafe fn list_to_str(list: *mut Self::ListHandle) -> *mut std::os::raw::c_char {
                    barvinok_sys::[<isl_ $prefix _list_to_str>](list)
                }

                unsafe fn printer_print_list(
                    p: *mut barvinok_sys::isl_printer,
                    list: *mut Self::ListHandle,
                ) -> *mut barvinok_sys::isl_printer {
                    barvinok_sys::[<isl_printer_print_ $prefix _list>](p, list)
                }

                unsafe fn list_dump(list: *mut Self::ListHandle) {
                    barvinok_sys::[<isl_ $prefix _list_dump>](list)
                }

                unsafe fn list_map(
                    list: *mut Self::ListHandle,
                    fn_ptr: unsafe extern "C" fn(*mut Self::Handle, *mut std::ffi::c_void) -> *mut Self::Handle,
                    user: *mut std::ffi::c_void,
                ) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_map>](list, Some(fn_ptr), user)
                }

                unsafe fn list_sort(
                    list: *mut Self::ListHandle,
                    cmp: unsafe extern "C" fn(*mut Self::Handle, *mut Self::Handle, *mut std::ffi::c_void) -> i32,
                    user: *mut std::ffi::c_void,
                ) -> *mut Self::ListHandle {
                    barvinok_sys::[<isl_ $prefix _list_sort>](list, Some(cmp), user)
                }

                unsafe fn list_foreach_scc(
                    list: *mut Self::ListHandle,
                    follows: unsafe extern "C" fn(*mut Self::Handle, *mut Self::Handle, *mut std::ffi::c_void) -> i32,
                    follows_user: *mut std::ffi::c_void,
                    fn_ptr: unsafe extern "C" fn(*mut Self::ListHandle, *mut std::ffi::c_void) -> i32,
                    fn_user: *mut std::ffi::c_void,
                ) -> i32 {
                    barvinok_sys::[<isl_ $prefix _list_foreach_scc>](list, Some(follows), follows_user, Some(fn_ptr), fn_user)
                }
            }
        }
    };
}

impl_list_raw_api!(
    crate::value::Value<'_>,
    handle = barvinok_sys::isl_val,
    list_handle = barvinok_sys::isl_val_list,
    prefix = val,
    unsafe fn get_handle(&self) -> *mut Self::Handle {
        self.handle.as_ptr()
    },
    unsafe fn from_raw_handle(handle: NonNull<Self::Handle>) -> Self {
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
);

impl_list_raw_api!(
    crate::constraint::Constraint<'_>,
    handle = barvinok_sys::isl_constraint,
    list_handle = barvinok_sys::isl_constraint_list,
    prefix = constraint,
    unsafe fn get_handle(&self) -> *mut Self::Handle {
        self.handle.as_ptr()
    },
    unsafe fn from_raw_handle(handle: NonNull<Self::Handle>) -> Self {
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
);

pub struct List<'a, T: ListRawAPI> {
    pub(crate) handle: NonNull<T::ListHandle>,
    pub(crate) marker: std::marker::PhantomData<*mut &'a [&'a T]>,
}

impl<'a, T: ListRawAPI + 'a> List<'a, T> {
    pub fn new(ctx: ContextRef<'a>, capacity: usize) -> Self {
        let handle = unsafe { T::list_alloc(ctx.0.as_ptr(), capacity as i32) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }

    pub fn context(&self) -> ContextRef<'a> {
        ContextRef(
            unsafe { NonNull::new_unchecked(T::get_context(self.handle.as_ptr())) },
            std::marker::PhantomData,
        )
    }

    pub fn len(&self) -> usize {
        unsafe { T::list_size(self.handle.as_ptr()) as usize }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn new_singleton(el: T) -> Self {
        let el = ManuallyDrop::new(el);
        let handle = unsafe { T::list_from_el(T::get_handle(&el)) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }

    pub fn push(&mut self, el: T) {
        let el = ManuallyDrop::new(el);
        let handle = unsafe { T::list_add(self.handle.as_ptr(), T::get_handle(&el)) };
        let handle = nonnull_or_alloc_error(handle);
        self.handle = handle;
    }

    pub fn dump(&self) {
        unsafe { T::list_dump(self.handle.as_ptr()) }
    }

    pub fn get(&self, index: usize) -> Option<T> {
        let el = unsafe { T::list_get_at(self.handle.as_ptr(), index as i32) };
        NonNull::new(el).map(|el| unsafe { T::from_raw_handle(el) })
    }

    pub fn set(&mut self, index: usize, el: T) {
        let el = ManuallyDrop::new(el);
        let handle =
            unsafe { T::list_set_at(self.handle.as_ptr(), index as i32, T::get_handle(&el)) };
        let handle = nonnull_or_alloc_error(handle);
        self.handle = handle;
    }

    pub fn iter<'u>(&'u self) -> Iter<'u, 'a, T> {
        Iter::new(self)
    }
}

impl<'a, T: ListRawAPI + 'a> Drop for List<'a, T> {
    fn drop(&mut self) {
        unsafe { T::list_free(self.handle.as_ptr()) };
    }
}

impl<'a, T: ListRawAPI + 'a> Clone for List<'a, T> {
    fn clone(&self) -> Self {
        let handle = unsafe { T::list_copy(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T: ListRawAPI + 'a> ISLPrint<'a> for List<'a, T> {
    type Handle = T::ListHandle;

    fn context(&self) -> ContextRef<'a> {
        self.context()
    }

    fn handle(&self) -> *mut Self::Handle {
        self.handle.as_ptr()
    }

    unsafe fn isl_printer_print(
        printer: *mut barvinok_sys::isl_printer,
        handle: *mut Self::Handle,
    ) -> *mut barvinok_sys::isl_printer {
        unsafe { T::printer_print_list(printer, handle) }
    }
}

impl<'a, T: ListRawAPI + 'a> std::fmt::Debug for List<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wrapper = crate::printer::FmtWrapper::new(self);
        std::fmt::Debug::fmt(&wrapper, f)
    }
}

pub struct Iter<'a, 'b, T: ListRawAPI + 'a> {
    list: &'a List<'b, T>,
    index: usize,
}

impl<'a, 'b, T: ListRawAPI + 'b> Iter<'a, 'b, T> {
    pub fn new(list: &'a List<'b, T>) -> Self {
        Self { list, index: 0 }
    }
}

impl<'a, 'b, T: ListRawAPI + 'b> Iterator for Iter<'a, 'b, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.list.len() {
            let el = self.list.get(self.index);
            self.index += 1;
            el
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.list.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, 'b, T: ListRawAPI + 'b> ExactSizeIterator for Iter<'a, 'b, T> {
    fn len(&self) -> usize {
        self.list.len() - self.index
    }
}

impl<'a, 'b, T: ListRawAPI + 'b> DoubleEndedIterator for Iter<'a, 'b, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index > 0 {
            self.index -= 1;
            self.list.get(self.index)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;
    use crate::value::Value;

    #[test]
    fn test_list_creation_and_push() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let mut list = List::<Value>::new(ctx, 10);
            assert_eq!(list.len(), 0);
            let val = Value::new_ui(ctx, 42);
            list.push(val.clone() + val.clone());
            list.push(val.clone() * val.clone());
            list.push(val);
            assert_eq!(list.len(), 3);
            list.dump();
        });
    }

    #[test]
    fn test_list_get() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let mut list = List::<Value>::new(ctx, 10);
            let val1 = Value::new_ui(ctx, 42);
            let val2 = Value::new_ui(ctx, 43);
            list.push(val1.clone());
            list.push(val2.clone());
            assert!(list.get(0).unwrap() == val1);
            assert!(list.get(1).unwrap() == val2);
            assert!(list.get(2).is_none());
        });
    }

    #[test]
    fn test_list_set() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let mut list = List::<Value>::new(ctx, 10);
            let val1 = Value::new_ui(ctx, 42);
            let val2 = Value::new_ui(ctx, 43);
            list.push(val1.clone());
            list.push(val2.clone());
            assert!(list.get(0).unwrap() == val1);
            assert!(list.get(1).unwrap() == val2);
            let val3 = Value::new_ui(ctx, 44);
            list.set(0, val3.clone());
            assert!(list.get(0).unwrap() == val3);
        });
    }
}
