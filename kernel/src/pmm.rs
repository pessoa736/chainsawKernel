
use alloc::vec::Vec;
use limine::memmap::MEMMAP_USABLE;
use limine::request::MemmapResponse;
use spin::Mutex;

use crate::debuglog::{dprint, dwrite};


pub const FRAME_SIZE: usize = 4096;

fn frame_floor(addr: usize) -> usize {
    addr / FRAME_SIZE
}

fn frame_ceil(addr: usize) -> usize {
    (addr + FRAME_SIZE - 1) / FRAME_SIZE
}

struct BitmapFrameAllocator {
    bitmap: Vec<u8>,
    total_frames: usize,
}

impl BitmapFrameAllocator {
    fn set_used(&mut self, frame: usize) {
        self.bitmap[frame / 8] |= 1 << (frame % 8);
    }

    fn set_free(&mut self, frame: usize) {
        self.bitmap[frame / 8] &= !(1 << (frame % 8));
    }

    fn is_free(&self, frame: usize) -> bool {
        (self.bitmap[frame / 8] & (1 << (frame % 8))) == 0
    }

    fn allocate_frame(&mut self) -> Option<usize> {
        for frame in 0..self.total_frames {
            if self.is_free(frame) {
                self.set_used(frame);
                return Some(frame * FRAME_SIZE);
            }
        }
        None
    }

    fn free_frame(&mut self, phys_addr: usize) {
        self.set_free(phys_addr / FRAME_SIZE);
    }
}

static PMM: Mutex<Option<BitmapFrameAllocator>> = Mutex::new(None);

// Assinatura de teste: recebe o response diretamente.
// Na versão real, isso vira `crate::memory::MEMORY_MAP_REQUEST.response().expect(...)`
// pub fn print_str(s: &[u8]) {
//     unsafe {
//         for &c in s {
//             core::arch::asm!("out dx, al", in("dx") 0x3f8u16, in("al") c, options(nomem, preserves_flags, nostack));
//         }
//     }
// }



pub fn fhex(mut valor: usize, buffer: &mut [u8; 16]) -> &str {
    if valor == 0 {
        buffer[15] = b'0';
        return core::str::from_utf8(&buffer[15..]).unwrap_or("0");
    }

    const HEX_MAP: &[u8; 16] = b"0123456789ABCDEF";
    let mut idx = buffer.len();

    while valor > 0 && idx > 0 {
        idx -= 1;
        buffer[idx] = HEX_MAP[valor & 0xF]; // Isola os últimos 4 bits (1 dígito hex)
        valor >>= 4;                        // Move 4 bits para a direita
    }

    core::str::from_utf8(&buffer[idx..]).unwrap_or("")
}



pub fn init_with_response(response: &MemmapResponse, heap_phys_base: usize, heap_size: usize) {
    dprint(Some("PMM.1"), "Entrou na funcao");

    let mut highest_addr: u64 = 0;
    for entry in response.entries() {
        if entry.type_ == limine::memmap::MEMMAP_USABLE
            || entry.type_ == limine::memmap::MEMMAP_BOOTLOADER_RECLAIMABLE
            || entry.type_ == limine::memmap::MEMMAP_EXECUTABLE_AND_MODULES
        {
            let end = entry.base + entry.length;
            if end > highest_addr {
                highest_addr = end;
            }
        }
    }

    let total_frames = frame_ceil(highest_addr as usize);
    let bitmap_bytes = (total_frames + 7) / 8;
    
    // Imprime o tamanho exato do bitmap para termos a certeza absoluta
    dprint(Some("PMM.2"),"Calculou bitmap_bytes: ");
    
    let mut _buffer = [0u8; 16];
    dwrite("0x");
    dwrite(fhex(bitmap_bytes,  &mut _buffer));

    let _teste_heap = alloc::boxed::Box::new(42_u64);
    dprint(Some("PMM.3"),"Box (Heap minimo) OK!");

    dprint(Some("PMM.3+1/2"),"Alocando vetor do Bitmap manualmente...");
    
    // A MAGIA ACONTECE AQUI: Burlamos a chamada ao `memset` do compilador
    let mut bitmap = alloc::vec::Vec::with_capacity(bitmap_bytes);
    unsafe {
        bitmap.set_len(bitmap_bytes);
        let ptr: *mut u8 = bitmap.as_mut_ptr();
        for i in 0..bitmap_bytes {
            // Escrita volátil impede o LLVM de otimizar isto para um `memset`
            core::ptr::write_volatile(ptr.add(i), 0xFF);
        }
    }

    let mut allocator = BitmapFrameAllocator {
        bitmap,
        total_frames,
    };
    dprint(Some("PMM.4"),"Vec do Bitmap OK!");

    for entry in response.entries() {
        if entry.type_ == limine::memmap::MEMMAP_USABLE {
            let start = frame_ceil(entry.base as usize);
            let end = frame_floor((entry.base + entry.length) as usize);
            for frame in start..end.min(allocator.total_frames) {
                allocator.set_free(frame);
            }
        }
    }
    dprint(Some("PMM.5"),"Regioes livres configuradas!");

    let heap_start = frame_floor(heap_phys_base);
    let heap_end = frame_ceil(heap_phys_base + heap_size);
    for frame in heap_start..heap_end.min(allocator.total_frames) {
        allocator.set_used(frame);
    }

    *PMM.lock() = Some(allocator);
    dprint(Some("PMM.6"),"Tudo pronto e salvo!");
}

pub fn allocate_frame() -> Option<usize> {
    PMM.lock()
        .as_mut()
        .expect("pmm::allocate_frame chamado antes de pmm::init")
        .allocate_frame()
}

pub fn free_frame(phys_addr: usize) {
    PMM.lock()
        .as_mut()
        .expect("pmm::free_frame chamado antes de pmm::init")
        .free_frame(phys_addr);
}