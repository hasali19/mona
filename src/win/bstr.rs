use winapi::{
    shared::wtypes::BSTR,
    um::oleauto::{SysAllocStringLen, SysFreeString},
};

use super::Variant;

pub struct BStr(BSTR);

impl BStr {
    pub fn from_str(str: &str) -> BStr {
        let chars: Vec<_> = str.encode_utf16().collect();
        let ptr = unsafe { SysAllocStringLen(chars.as_ptr(), chars.len() as _) };
        BStr(ptr)
    }

    pub fn inner(&self) -> BSTR {
        self.0
    }

    pub fn as_variant(&self) -> Variant<Self> {
        Variant::bstr(self)
    }
}

impl From<BStr> for BSTR {
    fn from(BStr(ptr): BStr) -> Self {
        ptr
    }
}

impl Drop for BStr {
    fn drop(&mut self) {
        unsafe { SysFreeString(self.0) }
    }
}
