use crate::video::{self, MAX_HEIGHT, MAX_WIDTH, clear_screen, unpack_color};

const FONTSIZE: usize = 16;
const PANDDING: usize = 5;
const MAXCOLS: usize = (MAX_WIDTH - PANDDING*2) / FONTSIZE;
const MAXROWS: usize = (MAX_HEIGHT - PANDDING*2) / FONTSIZE;

struct Terminal 
{
    cursor_x: usize,
    cursor_y: usize,
    cols: usize,
    rows: usize,
    fg_color: u32,
    bg_color: u32,
    buffer: [[u8; MAXCOLS]; MAXROWS]
}



static mut TERMINAL: Terminal = Terminal {
    cursor_x: 0,
    cursor_y: 0,
    cols: 0,
    rows: 0,
    fg_color: 0x883333,
    bg_color: 0xece8ba,
    buffer: [[b' '; MAXCOLS]; MAXROWS],
};


pub fn init()
{
    unsafe {
        TERMINAL.cols = video::get_width()/FONTSIZE;
        TERMINAL.rows = video::get_height()/FONTSIZE;
    }
    self::clear();
}


pub fn clear()
{
    unsafe {
        TERMINAL.cursor_x = 0;
        TERMINAL.cursor_y = 0;

        for r in 0..TERMINAL.rows {
            for c in 0..TERMINAL.cols {
                TERMINAL.buffer[r][c] = b' ';
            }
        }
    }

}

