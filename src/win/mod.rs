mod bstr;
mod variant;

pub mod taskschd;

use winapi::{shared::winerror::FAILED, um::winbase::GetUserNameW};

pub use bstr::BStr;
pub use variant::Variant;

pub fn get_user_name() -> anyhow::Result<String> {
    let mut buf = [0; 256];
    let mut len = buf.len() as u32;

    if FAILED(unsafe { GetUserNameW(buf.as_mut_ptr(), &mut len) }) {
        return Err(anyhow::anyhow!("failed to get username"));
    }

    Ok(String::from_utf16_lossy(&buf[..len as _]))
}
