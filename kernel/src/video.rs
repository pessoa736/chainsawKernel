use limine::request::FramebufferRequest;
use crate::db_font;
use crate::lerp_int;

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

static mut FRAMEBUFFER_PIXELS: *mut u32 = core::ptr::null_mut();

static mut PITCH_PIXELS: usize = 0;
static mut WIDTH: usize = 0;
static mut HEIGHT: usize = 0;

static mut RED_SHIFT: u8 = 0;
static mut RED_SIZE: u8 = 0;
static mut GREEN_SHIFT: u8 = 0;
static mut GREEN_SIZE: u8 = 0;
static mut BLUE_SHIFT: u8 = 0;
static mut BLUE_SIZE: u8 = 0;

pub const MAX_WIDTH: usize = 1920;
pub const MAX_HEIGHT: usize = 1080;
static mut BACKBUFFER: [u32; MAX_WIDTH * MAX_HEIGHT] = [0; MAX_WIDTH * MAX_HEIGHT];


#[derive(Copy, Clone, Debug)]
struct ColorChannels {
    r: u8,
    g: u8,
    b: u8,
    a: u8
}


pub fn init() {
    let mut to: u8 = 0;
    let mut initialized: bool = false;
    
    while !initialized {
        if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.response() {
            if let Some(framebuffer) = framebuffer_response.framebuffers().first() {
                unsafe {
                    FRAMEBUFFER_PIXELS = framebuffer.address() as *mut u32;
                    PITCH_PIXELS = (framebuffer.pitch as usize) / 4;
                    WIDTH = framebuffer.width as usize;
                    HEIGHT = framebuffer.height as usize;

                    if WIDTH > MAX_WIDTH { WIDTH = MAX_WIDTH; }
                    if HEIGHT > MAX_HEIGHT { HEIGHT = MAX_HEIGHT; }

                    RED_SHIFT = framebuffer.red_mask_shift;
                    RED_SIZE = framebuffer.red_mask_size;
                    GREEN_SHIFT = framebuffer.green_mask_shift;
                    GREEN_SIZE = framebuffer.green_mask_size;
                    BLUE_SHIFT = framebuffer.blue_mask_shift;
                    BLUE_SIZE = framebuffer.blue_mask_size;

                    initialized = true;
                }
            }
        }

        if to > 100 || initialized { 
            break;
        }
        to = to + 1;
    }
}

pub fn pack_color(r: u8, g: u8, b: u8, alpha: u8) -> u32 
{
    ((alpha as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | ((b as u32)) 
}

pub fn unpack_color(color: u32) -> ColorChannels 
{
    ColorChannels { 
        a: ((color>>24) & 0xFF ) as u8,
        r: ((color>>16) & 0xFF ) as u8,
        g: ((color>>8) & 0xFF ) as u8,
        b: ((color) & 0xFF ) as u8,
    }
}


pub fn hardware_pack_color(r: u8, g: u8, b: u8) -> u32 {
    unsafe {
        let rv = if RED_SIZE == 0 { 0 } else { (r as u32) >> (8 - RED_SIZE) };
        let gv = if GREEN_SIZE == 0 { 0 } else { (g as u32) >> (8 - GREEN_SIZE) };
        let bv = if BLUE_SIZE == 0 { 0 } else { (b as u32) >> (8 - BLUE_SIZE) };

        (rv << RED_SHIFT) | (gv << GREEN_SHIFT) | (bv << BLUE_SHIFT)
    }
}

pub fn set_pixel(x: usize, y: usize, color: u32) {
    unsafe {
        if x >= WIDTH || y >= HEIGHT {
            return;
        }

        let src = unpack_color(color);

        if src.a == 0 {return;}

        if src.a == 255 {
            BACKBUFFER[y * MAX_WIDTH + x] = color;
            return;
        }

        let dest = unpack_color(BACKBUFFER[y * MAX_WIDTH + x]);
        let a = src.a as u128;
        let r_res = ((src.r as u128 * a + dest.r as u128 * (255 - a)) / 255) as u8;
        let g_res = ((src.g as u128 * a + dest.g as u128 * (255 - a)) / 255) as u8;
        let b_res = ((src.b as u128 * a + dest.b as u128 * (255 - a)) / 255) as u8;

        BACKBUFFER[y*MAX_WIDTH + x] = pack_color(r_res, g_res, b_res, 255);
    }
}

pub fn swap_buffers() {
    unsafe {
        if FRAMEBUFFER_PIXELS.is_null() {
            return;
        }
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let cor = BACKBUFFER[y * MAX_WIDTH + x];

                let canais = unpack_color(cor);
                let cor_hardware = hardware_pack_color(canais.r, canais.g, canais.b);

                FRAMEBUFFER_PIXELS.add(y * PITCH_PIXELS + x).write_volatile(cor_hardware);
            }
        }
    }
}

pub fn clear_screen(color: u32) {
    unsafe {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                BACKBUFFER[y * MAX_WIDTH + x] = color;
            }
        }
    }
}

fn get_emoji_index(c: char) -> Option<usize> {
    match c {
        '🎀' => Some(1),
        '💖' => Some(3),
        _ => None
    }
}

pub fn draw_char(x: usize, y: usize, c: char, color: u32) {
    let glyph: [u16; 16];

    if let Some(emoji_idx) = get_emoji_index(c) {
        glyph = db_font::FONT_EMOJI[emoji_idx];
    } else {
        let ci = c as usize;
        if ci >= 128 {
            return;
        }
        glyph = db_font::FONT_TEXT[ci];
    }
    
    for row in 0..16 {
        let bits = glyph[row];
        for col in 0..16 {
            if (bits & (1 << (15 - col))) != 0 {
                set_pixel(x + col, y + row, color);
            }
        }
    }
}

pub fn draw_string(mut x: usize, y: usize, s: &str, color: u32) {
    for c in s.chars() {
        draw_char(x, y, c, color);
        x += 16;
    }
}

pub fn get_width() -> usize {
    unsafe { WIDTH }
}

pub fn get_height() -> usize {
    unsafe { HEIGHT }
}


pub fn lerp_color(cor_a: u32, cor_b: u32, t: u8) -> u32 {
    let c_a = unpack_color(cor_a);
    let c_b = unpack_color(cor_b);

    let weight = t as u128;

    let r_res = ((c_a.r as u128 * (255 - weight) + c_b.r as u128 * weight) / 255) as u32;
    let g_res = ((c_a.g as u128 * (255 - weight) + c_b.g as u128 * weight) / 255) as u32;
    let b_res = ((c_a.b as u128 * (255 - weight) + c_b.b as u128 * weight) / 255) as u32;
    let a_res = ((c_a.a as u128 * (255 - weight) + c_b.a as u128 * weight) / 255) as u32;

    pack_color(r_res as u8, g_res as u8, b_res as u8, a_res as u8)
}