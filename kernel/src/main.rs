#![no_std]
#![no_main]

use limine::request::StackSizeRequest;
use limine::BaseRevision;
use core::panic::PanicInfo;

use crate::video::{pack_color, swap_buffers};

mod memory;
mod video;
mod db_font;
mod io;
mod math;
mod terminal;
mod tensors;


#[used]
#[link_section = ".requests"]
static STACK_REQUEST: StackSizeRequest = StackSizeRequest::new(2 * 1024 * 1024);

#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();


pub fn init_kernel() 
{
    memory::init();
    video::init();
    io::init();   
    terminal::init();
}


fn intro(){ 
    let wid = video::get_width();
    let hei = video::get_height();
    
    let mut t: f64 = 0.0;
    unsafe 
    {
        video::clear_screen(video::pack_color(240, 10, 10, 255));
        for y in 0 .. (hei/2) 
        {
            for x in 0 .. (wid/2) 
            {
                video::set_pixel(x + wid/4, y+hei/4, video::lerp_color(pack_color(0, 0, 0, 0), 0xfff8f7c7, t as u8));
            }
        }


        video::draw_string(wid/2, hei/2, "test aaaaaaaa", 0x000);
        video::draw_string(wid/2, hei/2 + 8, "TEST 💖", 0x000);

        t+=0.05;
    }
}


#[no_mangle]
pub extern "C" fn _start() -> ! 
{
    assert!(BASE_REVISION.is_supported());

    init_kernel();


    terminal::write_str("hello world!\n\nwelcome to Motossera Kernel!\n\n");
    
   

    loop {
        let value = io::get_keyboard_input();
        if value != 0 {terminal::write_char(value);}
        terminal::render();
        swap_buffers();    
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
