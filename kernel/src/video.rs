use core::{cell::UnsafeCell, time::Duration };
use limine::{framebuffer::{Framebuffer}, request::FramebufferRequest};
use na::{Const, Matrix, SMatrix, SMatrixViewMut, ViewStorageMut};
use naive_timer::Timer;
use spin::{LazyLock, Mutex};
use crate::{db_font, debuglog::dprint, timer::{SubTimer, TIMER, TimeInteration}};
use core::mem::MaybeUninit;



/// matriz de cor binaria:
///  
/// ```
/// | color | shift | size |
/// | red   |   r1  |  r2  |
/// | green |   g1  |  g2  |
/// | blue  |   b1  |  b2  |
/// ```
/// 
pub struct MaskColor
{ rgb: UnsafeCell<SMatrix<u8, 3,2>> }

type ColorRow = SMatrix<u8, 1, 2>;

impl MaskColor 
{
    
    #[inline(always)]
    pub fn get_rgb(&mut self) -> &mut SMatrix<u8, 3, 2>
    { 
        unsafe {&mut *self.rgb.get()} 
    }

    #[inline(always)]
    pub fn get_sizes(&mut self) -> SMatrixViewMut<'_, u8, 3, 1>
    {self.get_rgb().column_mut(1)}


    #[inline(always)]
    pub fn get_shifts(&mut self) -> SMatrixViewMut<'_, u8, 3, 1>
    {self.get_rgb().column_mut(0)}

    #[inline(always)]
    pub fn get_red(&mut self) -> Matrix<u8, Const<1>, Const<2>, ViewStorageMut<'_, u8, Const<1>, Const<2>, Const<1>, Const<3>>>
    {self.get_rgb().row_mut(0)}

    #[inline(always)]
    pub fn get_green(&mut self) -> Matrix<u8, Const<1>, Const<2>, ViewStorageMut<'_, u8, Const<1>, Const<2>, Const<1>, Const<3>>>
    {self.get_rgb().row_mut(1)}

    #[inline(always)]
    pub fn get_blue(&mut self) -> Matrix<u8, Const<1>, Const<2>, ViewStorageMut<'_, u8, Const<1>, Const<2>, Const<1>, Const<3>>>
    {self.get_rgb().row_mut(2)}

    

    /// ### criar uma nova mascara
    /// 
    /// ```rust 
    /// MaskColor::New() === MaskColor { rgb: UnsafeCell<SMatrix<u8, 3, 2>> }
    /// 
    /// // exemplo
    /// static CM: LazyLock<MaskColor> = LazyLock::new(||{
    ///     MaskColor::new()
    /// })
    /// ```
    /// 
    #[inline(always)]
    pub fn new() -> Self 
    {Self{ rgb: UnsafeCell::new(SMatrix::zeros()) }} 



    pub fn set_with_limine_framebuffer(&mut self, framebuffer: &&Framebuffer)
    {
        let colors = self.get_rgb();

        colors.set_row(0, &ColorRow::new(framebuffer.red_mask_shift, framebuffer.red_mask_size));
        colors.set_row(1, &ColorRow::new(framebuffer.green_mask_shift, framebuffer.green_mask_size));
        colors.set_row(2, &ColorRow::new(framebuffer.blue_mask_shift, framebuffer.blue_mask_size));
    }



    pub fn pack(&mut self, rgb: [u8; 3]) -> u32 
    {
        let colors = self.get_rgb(); 

        let mut or_: u32 = 0;
        for i in 0..3 
        {
            let size = unsafe {*colors.get_unchecked((i, 1))};
            let shift = unsafe {*colors.get_unchecked((i, 0))};

            if size != 0 { or_ |= ((rgb[i] as u32) >> (8 - size)) << shift; }
        }
        or_
    }

}


unsafe impl Sync for MaskColor 
{}

pub static MASKCOLOR: LazyLock<MaskColor> = LazyLock::new(| | {MaskColor::new()});



/// matriz de cor virtual:
/// 
/// ```
/// | color | shift | size |
/// | red   |   r1  |  r2  |
/// | green |   g1  |  g2  |
/// | blue  |   b1  |  b2  |
/// | alfa  |   a1  |  a2  |
/// ```
/// 

#[derive(Copy, Clone, Debug)]
pub struct VirtualColor 
{ 
    r: u8,
    g: u8,
    b: u8,
    a: u8
}


impl VirtualColor 
{
    #[inline(always)]
    pub fn new_hex(color : u32) -> Self
    {
        Self{
            r: ((color >> 24) & 0xFF) as u8,
            g: ((color >> 16) & 0xFF) as u8,
            b: ((color >> 8) & 0xFF) as u8,
            a: (color & 0xFF) as u8,
        }
    }


    #[inline(always)]
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self
    {
        Self{
            r: r,
            g: g,
            b: b,
            a: a,
        }
    }    

    pub fn unpack(&self) -> u32 
    {
        ((self.a as u32) << 24) | 
        ((self.r as u32) << 16) | 
        ((self.g as u32) << 8) | 
        (self.b as u32)
    }

    pub fn lerp(&self, color2: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let um_menos_t = 1.0 - t;

        Self {
            r: ((self.r as f32 * um_menos_t) + (color2.r as f32 * t)) as u8,
            g: ((self.g as f32 * um_menos_t) + (color2.g as f32 * t)) as u8,
            b: ((self.b as f32 * um_menos_t) + (color2.b as f32 * t)) as u8,
            a: ((self.a as f32 * um_menos_t) + (color2.a as f32 * t)) as u8,
        }
    }
}



pub const MAX_WIDTH: usize = 1920;
pub const MAX_HEIGHT: usize = 1080;



// Wrapper seguro ao redor da SMatrix do nalgebra
pub struct FramebufferWrapper<const H: usize, const W: usize> 
{ mat: UnsafeCell<SMatrix<u32, H, W>> }


impl<const H: usize, const W: usize> FramebufferWrapper<H, W> 
{
    
    #[inline(always)]
    pub fn new() -> Self
    {Self{mat: UnsafeCell::new(SMatrix::zeros())}}

    #[inline(always)]
    pub fn get_mat(&mut self) -> &mut SMatrix<u32, H, W>
    {unsafe {&mut *self.mat.get()}}

    #[inline(always)]
    pub fn set(&mut self, row: usize, col: usize, val: u32) 
    {unsafe {*self.get_mat().get_unchecked_mut((row % H, col % W)) = val;}}

    #[inline(always)]
    pub fn get(&mut self, row: usize, col: usize) -> u32 
    {unsafe { *self.get_mat().get_unchecked((row % H, col % W)) }}

    #[inline(always)]
    pub fn clear(&mut self, color: u32) 
    {for pixel in self.get_mat().iter_mut() { *pixel = color; }}

}

// Permite que o wrapper seja instanciado como uma variável global static
unsafe impl<const H: usize, const W: usize> Sync for FramebufferWrapper<H, W> {}


pub struct Vga 
{
    framebuffer: Mutex<MaybeUninit<FramebufferWrapper<MAX_HEIGHT, MAX_WIDTH>>>,
    framebufferpixels: *mut u32,
    picthpixels: usize,
    width: usize, 
    height: usize
}

impl Vga {
    pub const fn new() -> Self
    {Self {
        framebuffer: Mutex::new(MaybeUninit::zeroed()),
        framebufferpixels: core::ptr::null_mut(),
        picthpixels: 0,
        width: 0,
        height: 0
    }}

    pub fn init(&mut self, fb_request: &FramebufferRequest) -> bool 
    {
        let mut timeout = TimeInteration::new(Some(100u64));
        
        let mut initialized: bool = false;
        dprint(Some("[V.0]"), "video::init começou");

        timeout.loop_(|_time| {
            dprint(Some("[V.1]"), "esperando request");
            if let Some(framebuffer_response) = fb_request.response() 
            {
                dprint(Some("[V.2]"), "recebeu a resposta da requisção");
                if let Some(framebuffer) = framebuffer_response.framebuffers().first() 
                {
                    dprint(Some("[V.3]"), "Recebeu o frame buffer");
                    unsafe 
                    {
                        self.framebufferpixels = framebuffer.address() as *mut u32;
                        self.picthpixels       = (framebuffer.pitch as usize) / 4;
                        self.width             = framebuffer.width as usize;
                        self.height            = framebuffer.height as usize;

                        if self.width > MAX_WIDTH { self.width = MAX_WIDTH; }
                        if self.height > MAX_HEIGHT { self.height = MAX_HEIGHT; }
                        dprint(Some("[V.4]"), "organizou as dimenções");

                        (&mut *MASKCOLOR.as_mut_ptr()).set_with_limine_framebuffer(framebuffer); 

                        dprint(Some("[V.5]"), "organizou as maskaras de cores");

                        // Limpa o backbuffer estático com preto no boot
                        //self.framebuffer.lock().clear(0x0);

                        initialized = true;
                        dprint(Some("[V.8]"), "iniciou");
                    }
                }
            }
            
            if initialized {_time.close();}
        });
        
        initialized
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        let mut guard_bf = self.framebuffer.lock();
        let fb = unsafe {guard_bf.assume_init_mut()};

        if x >= self.width || y >= self.height {
            return;
        }

        let src = VirtualColor::new_hex(color);
        if src.a == 0 { return; }

        // IMPORTANTE: nalgebra usa (linha, coluna) -> mapeamos para (y, x)
        if src.a == 255 {
            (*fb).set(x, y, color); 
            return;
        }

        let dest = VirtualColor::new_hex((*fb).get(y, x));

        let a = src.a as u128;
        let r_res = ((src.r as u128 * a + dest.r as u128 * (255 - a)) / 255) as u8;
        let g_res = ((src.g as u128 * a + dest.g as u128 * (255 - a)) / 255) as u8;
        let b_res = ((src.b as u128 * a + dest.b as u128 * (255 - a)) / 255) as u8;

        let nc = VirtualColor::new(r_res, g_res, b_res, 0xff);

        (*fb).set(y, x, nc.unpack());
    }

    pub fn swap_buffers(&mut self) 
    {
        unsafe 
        {
            if self.framebufferpixels.is_null() 
            { return; }

            let mut guard_bf = self.framebuffer.lock();
            let fb = unsafe { guard_bf.assume_init_mut() };

            for y in 0..self.height 
            { 
                for x in 0..self.width 
                {
                    // Recupera a cor do nosso backbuffer nalgebra
                    let cor = (*fb).get(y, x);


                    // Escrita volátil direta na memória de vídeo mapeada pelo Limine
                    self.framebufferpixels
                        .add(y * self.picthpixels + x)
                        .write_volatile(cor);
                }
            }
        }
    }

    pub fn clear(&mut self, color: u32) 
    {
        let mut guard_bf = self.framebuffer.lock();
        let fb = unsafe {guard_bf.assume_init_mut()};
        (*fb).clear(color);
    }

    pub fn draw_char(&mut self, x: usize, y: usize, c: char, color: u32) 
    {
        let glyph: [u16; 16];

        // if let Some(emoji_idx) = get_emoji_index(c) {
        //     glyph = db_font::FONT_EMOJI[emoji_idx];
        // } else {
            let ci = c as usize;
            if ci >= 128 {
                return;
            }
            glyph = db_font::FONT_TEXT[ci];
        // }
        
        for row in 0..16 
        {
            let bits = glyph[row];
            for col in 0..16 
            {
                if (bits & (1 << (15 - col))) != 0 
                {
                    self.set_pixel(x + col, y + row, color);
                }
            }
        }
    }
    
    pub fn draw_string(&mut self,mut x: usize, y: usize, s: &str, color: u32) 
    {
        for c in s.chars() 
        {
            self.draw_char(x, y, c, color);
            x += 16;
        }
    }


    pub fn draw_rect(&mut self, pos: (usize, usize), size: (usize, usize), color: u32)
    {
        let mut guard_bf = self.framebuffer.lock();
        let fb = unsafe { guard_bf.assume_init_mut() };

        for pixel in fb
            .get_mat()
            .view_mut((pos.1, pos.0), (size.1, size.0))
            .iter_mut()
        {
            *pixel = color;
        }

        drop(guard_bf);
    }


    pub fn get_width(&self) -> usize {
        self.width 
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

}





pub const RED_COLOR: u32 = 0xFFF01010;
pub const OFFWHITE_COLOR: u32 = 0xFFece8ba;