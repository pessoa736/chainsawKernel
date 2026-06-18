use core::arch::asm;



static COM1: u16 = 0x3f8;
const KEYBOARD_STATUS: u16 = 0x64;
const KEYBOARD_DATA: u16 = 0x60;




#[inline]
unsafe fn inb(port: u16) -> u8 
{
    let ret: u8;
    asm!(
        "in al, dx",
        in("dx") port,
        out("al") ret,
        options(nomem, nostack, preserves_flags)
    );
    ret
}



#[inline]
unsafe fn outb(port: u16, data: u8) 
{
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") data,
        options(nomem, nostack, preserves_flags)
    );
}


pub fn init()
{
    unsafe {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1 + 0, 0x03);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0x07);
        outb(COM1 + 4, 0x0b);
    };
}

pub fn ite() -> bool {
    unsafe {
        (inb(COM1 + 5) & 0x20) != 0
    }
}

pub fn get_keyboard_input() -> u8 {
    unsafe {
        let status = inb(KEYBOARD_STATUS);

        if (status & 0x01) != 0 
        { 
            return inb(KEYBOARD_DATA);
        }
        
        0
    }
}


pub fn pcs(c: u8)
{
    while !ite() {}

    unsafe 
    {
        outb(COM1, c);
    };
}

pub fn pss(s: &str){
    for byte in s.bytes() {
        pcs(byte);
    }
}