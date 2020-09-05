use std::marker::PhantomData;

use winapi::{
    shared::wtypes::{VARENUM, VT_BSTR, VT_NULL},
    um::oaidl::{VARIANT_n3, VARIANT},
};

use super::BStr;

pub struct Variant<'a, T: 'a>(VARIANT, PhantomData<&'a T>);

impl<'a, T> Variant<'a, T> {
    fn new() -> Self {
        Variant(VARIANT::default(), PhantomData::default())
    }

    pub unsafe fn inner(&self) -> VARIANT {
        self.0
    }

    unsafe fn with_type(mut self, t: VARENUM) -> Self {
        self.0.n1.n2_mut().vt = t as _;
        self
    }

    unsafe fn with_value<U>(mut self, f: impl Fn(&mut VARIANT_n3) -> &mut U, v: U) -> Self {
        *f(&mut self.0.n1.n2_mut().n3) = v;
        self
    }
}

impl<'a> Variant<'a, ()> {
    pub fn null() -> Self {
        unsafe { Variant::new().with_type(VT_NULL) }
    }
}

impl<'a> Variant<'a, BStr> {
    pub fn bstr(bstr: &'a BStr) -> Self {
        unsafe {
            Variant::new()
                .with_type(VT_BSTR)
                .with_value(|n3| n3.bstrVal_mut(), bstr.inner())
        }
    }
}
