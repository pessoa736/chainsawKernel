
use core::{cell::UnsafeCell, mem::MaybeUninit};
use na::{SMatrix, Scalar};
use spin::{Mutex};
use crate::{debuglog::{dprint, dwrite}, video::{MAX_HEIGHT, MAX_WIDTH, OCEANDARK_COLOR, OFFWHITE_COLOR, Vga, VirtualColor}};



const MAXCOLS: usize = MAX_WIDTH / 8; 
const MAXROWS: usize = MAX_HEIGHT / 8;

#[derive(Clone, Debug)]
struct Cell {
    color: VirtualColor,
    background_color: VirtualColor,
    _char: u8,
}

impl Cell {
    pub const fn new() -> Self 
    {Self { 
        color: VirtualColor::new_hex(0x0), 
        background_color: VirtualColor::new_hex(0x0), 
        _char: b'0' 
    }}
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool 
    { if 
        self._char == other._char && 
        self.color == other.color && 
        self.background_color == other.background_color 
        {true} else {false} 
    }
}



struct Wrapperbuff<const H: usize, const W: usize> {
    mat: UnsafeCell<SMatrix<Cell, H, W>>
}

unsafe impl<const H: usize, const W: usize> Sync for Wrapperbuff<H, W> {}

impl<const H: usize, const W: usize> Wrapperbuff<H, W> {
    
    #[inline(always)]
    pub fn new() -> Self
    {Self{mat: UnsafeCell::new(SMatrix::from_fn(| _, _ | Cell::new()))}}
}


pub struct Terminal 
{
    FONTSIZE: usize,
    PANDDING: usize,

    pos_cursor: (usize, usize), // .0 = x, .1 = y
    aspect: (usize, usize), // .0 = row, .1 == cols
    fg_color: u32,
    bg_color: u32,
    buffer: Mutex<MaybeUninit<Wrapperbuff<MAXROWS, MAXCOLS>>>
}


impl Terminal {
    pub const fn new() -> Self 
    {Self { 
        FONTSIZE: 0, 
        PANDDING: 0, 
        pos_cursor: (0, 0), 
        aspect: (0, 0), 
        fg_color: 0x0, 
        bg_color: 0x0, 
        buffer: Mutex::new(MaybeUninit::zeroed())
    }}
    
    pub fn clear(&mut self) {
        self.pos_cursor = (0, 0);
        
        let mut guard = self.buffer.lock();
        let buf = unsafe { guard.assume_init_mut() };

        for cell in (&mut *(*buf).mat.get_mut()).iter_mut() { *cell = Cell::new(); }

        drop(guard);
    }

    pub fn init(&mut self, vga: &Vga) -> bool
    {

        dprint(Some("Term.0"), "terminal::init começou");

        self.PANDDING = 8;
        self.FONTSIZE = 8;


        dprint(Some("Term.1"), "ajeitou o tamanho da fonte e o pandding");

        self.aspect = (
            (vga.get_height() - self.PANDDING*2) / self.FONTSIZE,
            (vga.get_width() - self.PANDDING*2) / self.FONTSIZE
        );


        dprint(Some("Term.2"), "ajeitou o aspecto do terminal");
        
        self.bg_color = VirtualColor::new_hex(OCEANDARK_COLOR).unpack();
        self.fg_color = VirtualColor::new_hex(OFFWHITE_COLOR).unpack();


        dprint(Some("Term.3"), "ajeitou as cores");

        self.clear();
    
        true
    }

    pub fn scroll_up(&mut self)
    {
        let mut guard = self.buffer.lock();
        let buf_ptr = unsafe { guard.assume_init_mut() };
        let buff =  (*buf_ptr).mat.get_mut();

        for i in 1..self.aspect.0 
        { buff.swap_columns(i-1, i);}

        buff.fill_row(self.aspect.1, Cell::new());
        
        self.pos_cursor.1 = self.aspect.0 - 2;

        drop(guard);
    }

    pub fn write_char(&mut self, c: u8)
    {

        let mut guard = self.buffer.lock();
        let buf_ptr = unsafe { guard.assume_init_mut() };
        let buff =  (*buf_ptr).mat.get_mut();
      
        match c 
        {
            b'\n' => 
            {
                self.pos_cursor.0 = 0;
                self.pos_cursor.1 += 1;
            }

            b'\r' => 
            {
                self.pos_cursor.0 = 0;
            }

            b'\x08' => 
            {
                // Backspace
                if self.pos_cursor.0 > 0 
                {
                    self.pos_cursor.0 -= 1;
                    unsafe{*buff.get_unchecked_mut(self.pos_cursor) =  Cell::new();}
                
                } else if self.pos_cursor.1 > 0 
                {
                    self.pos_cursor.1 -= 1;
                    self.pos_cursor.0 = self.aspect.1 - 1;
                    unsafe{*buff.get_unchecked_mut(self.pos_cursor) =  Cell::new();}
                }
            }

            _ => 
            {
                if self.pos_cursor.0 < self.aspect.1 && self.pos_cursor.1 < self.aspect.0 
                {
                    unsafe{*buff.get_unchecked_mut(self.pos_cursor) =  Cell::new();}
                    self.pos_cursor.0 += 1;
                }
            }
        }

        if self.pos_cursor.1 >= self.aspect.0 - 1 {
            drop(guard);
            self.scroll_up();
        }
    }

    pub fn write_str(&mut self, s: &str)
    {
        for b in s.bytes() 
        { self.write_char(b); }
        dwrite(s);
    }

    pub fn render(&mut self,  vga: &mut Vga)
    {   
        vga.clear(self.bg_color);
        
        
        let mut guard = self.buffer.lock();
        let buf_ptr = unsafe { guard.assume_init_mut() };
        let buff =  (*buf_ptr).mat.get_mut();

        let _ = buff.map_with_location(
            |r, c, val| 
            {
                let pos = (
                    c*self.FONTSIZE + self.PANDDING,
                    r*self.FONTSIZE + self.PANDDING
                );

                if pos == self.pos_cursor 
                {
                    vga.draw_char(pos.0, pos.1, val._char as char, self.bg_color);
                }else {
                    vga.draw_char(pos.0, pos.1, val._char as char, self.fg_color);
                }

            }
        );

        drop(guard);
    }
    pub fn logic ()
    {
        
    }
}


