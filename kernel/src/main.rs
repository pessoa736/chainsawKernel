#![no_std]
#![no_main]
#![feature(generic_const_exprs)]
// #![feature(inherent_associated_types)]
#![allow(incomplete_features)]
// #![feature(abi_x86_interrupt)]

use core::time::Duration;

use limine::request::{FramebufferRequest, StackSizeRequest};
use limine::BaseRevision;
use spin::{Mutex};

use crate::terminal::Terminal;
use crate::timer::{TimeInteration};
use crate::{debuglog::dprint, timer::TIMER, video::{Vga, VirtualColor}};

extern crate alloc;
pub extern crate nalgebra as na;


mod debuglog;
mod memory;
mod pmm;
//mod lua;
mod video;
mod db_font;
mod io;
mod math;
mod terminal;
//mod matriz;
mod timer;
//mod shell;
mod stubs;


const SKIPINTRO: bool = true;

#[no_mangle]
#[used]
#[link_section = ".requests"]
static STACK_REQUEST: StackSizeRequest = StackSizeRequest::new(32 * 1024 * 1024);

#[no_mangle]
#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();


#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();


#[repr(align(16))]
struct Kernel {
    timer: TIMER,
    vga: Vga,
    terminal: Terminal
}


impl Kernel {
    pub fn init(&mut self) 
    {
        if !BASE_REVISION.is_supported() {panic!("NO_BASE\n")}
        dprint(Some("BOOT"), "B");

        io::init();
        dprint(Some("IK.1"), "io inicializou");
        
        let (heap_phys_base, heap_size) = memory::init();
        if let Some(memmap_response) = memory::MEMORY_MAP_REQUEST.response() {
            pmm::init_with_response(memmap_response, heap_phys_base, heap_size);
        } else {
            panic!("Falha ao obter memmap para o PMM");
        }
        dprint(Some("IK.2"), "Memoria inicializou");

        self.vga.init(&FRAMEBUFFER_REQUEST);
        dprint(Some("IK.3"), "video inicializou");

        self.terminal.init(&self.vga);
        dprint(Some("IK.4"), "terninal inicializou");
        // dprint(Some("BOOT"), "C");
    
        // paint_welcome();
        // dprint(Some("BOOT"), "D");
        //dprint(, s);

        // shell::init()
    }

    pub fn Loop(&mut self)
    {
        // let mut Looptime = TimeInteration::new(None);
        // dprint(Some("loop"), "inicializou o loop");
        
        // Looptime.loop_(
        //     |_t| {

        //     }
        // );
        loop {
            self.terminal.render(&mut self.vga);
            self.vga.swap_buffers();
        }
    }


    pub fn paint_welcome(&mut self)
    {
        let mut wel_time = TimeInteration::new(Some(Duration::from_secs(1).as_millis() as u64));


        dprint(Some("welcome"), "inicializou o welcome");

        let wid = self.vga.get_width();
        let hei = self.vga.get_height();

        dprint(Some("welcome"), "pegou as proporções da tela");

        self.vga.clear(VirtualColor::new(20, 20, 30, 255).unpack());
        
        dprint(Some("welcome"), "limpou a tela");

        let bg_i_c = VirtualColor::new_hex(0x0);
        let bg_f_c = VirtualColor::new_hex(0xcc2222ff);
        let fg_i_c = VirtualColor::new_hex(0x0);
        let fg_f_c = VirtualColor::new_hex(0xeeeeaaff);

        wel_time.loop_(|_time | 
        {
            let bg = bg_i_c.lerp(bg_f_c, _time.percent());
            let fg = fg_i_c.lerp(fg_f_c, _time.percent());

            self.vga.clear(bg.unpack());
            self.vga.draw_string(
                wid/2 - 32 * 8, hei/2,
                "BEM VINDO AO MOTOSSERA KERNEL", 
                fg.unpack()
            );
            // Linhas decorativas
            self.vga.draw_rect((wid/4, hei/3),(wid/2, 1), fg.unpack());
            self.vga.draw_rect((wid/4, hei - hei/3), (wid/2, 1), fg.unpack());

            self.vga.swap_buffers();
        });
        wel_time.reset();
        wel_time.loop_(|_time | 
        {
            let bg = bg_f_c.lerp(bg_i_c, _time.percent());
            let fg = fg_f_c.lerp(fg_i_c, _time.percent());

            self.vga.clear(bg.unpack());
            self.vga.draw_string(
                wid/2 - 32 * 8, hei/2,
                "BEM VINDO AO MOTOSSERA KERNEL", 
                fg.unpack()
            );
            // Linhas decorativas
            self.vga.draw_rect((wid/4, hei/3),(wid/2, 1), fg.unpack());
            self.vga.draw_rect((wid/4, hei - hei/3), (wid/2, 1), fg.unpack());

            self.vga.swap_buffers();
        });

    }
}

unsafe impl Sync for Kernel {}


/// Pinta uma tela única de boas-vindas no framebuffer. Chamada antes de
/// entrar no shell, para garantir que algo renderiza (sem depender de
/// loop de intro que nunca disparava o swap_buffers).

static mut KERNEL: Mutex<Kernel> = Mutex::new(
    Kernel {
        timer: TIMER,
        vga: Vga::new(),
        terminal: Terminal::new()
    }
);


#[no_mangle]
pub extern "C" fn _start() -> ()
{

    boot_marker();

    
    unsafe {
        let kernel = KERNEL.get_mut();
        kernel.init();
        if !SKIPINTRO {kernel.paint_welcome();};
        kernel.Loop();
    }

    loop {}
}

#[inline(never)]
fn boot_marker() {
    dprint(Some("BOOT"), "A" );
}
