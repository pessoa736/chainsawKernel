use core::arch::asm;
use core::panic::PanicInfo;
use core::fmt::{Write, Result};

struct SerialWriter;

impl Write for SerialWriter {


    #[inline(always)]
    fn write_str(&mut self, s: &str) -> Result {
        let com1: u16 = 0x3f8;
        unsafe {
            for byte in s.bytes() {
                asm!(
                    "out dx, al", 
                    in("dx") com1, 
                    in("al") byte, 
                    options(nomem, preserves_flags, nostack)
                );
            }
        }
        Ok(())
    }
}

#[inline(always)]
pub fn dwrite(s: &str) {
    let mut writer = SerialWriter;
    let _ = writer.write_str(s);
}

pub fn dprint(title: Option<&str>, s: &str) {
    if let Some(t) = title {
        dwrite("\n[ ");
        dwrite(t);
        dwrite(" ] ");
    }
    dwrite(s);
}


#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    let mut writer = SerialWriter;
    let _ = write!(writer, "\n[ panic ] ");
    let _ = write!(writer, "{}", _info);
    loop {}
}
