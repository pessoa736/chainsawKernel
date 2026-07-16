use limine::request::{MemmapRequest, HhdmRequest};
use limine::memmap::MEMMAP_USABLE;
use linked_list_allocator::LockedHeap;

#[no_mangle]
#[used]
#[link_section = ".requests"]
pub static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new();

#[no_mangle]
#[used]
#[link_section = ".requests"]
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Definimos um limite seguro para o heap do Kernel (ex: 32 MB)
const KERNEL_HEAP_SIZE_LIMIT: usize = 64 * 1024 * 1024; 

pub fn allocator_stats() -> (usize, usize) {
    let h = ALLOCATOR.lock();
    (h.size(), h.used())
}

// pub fn init() -> (usize, usize) {
//     let hhdm_offset = HHDM_REQUEST
//         .response()
//         .map(|r| r.offset)
//         .expect("HHDM request failed - bootloader nao respondeu");

//     let response = MEMORY_MAP_REQUEST
//         .response()
//         .expect("Nao foi possivel obter o Memory Map do Limine");

//     let mut largest_base = 0;
//     let mut largest_size = 0;

//     for entry in response.entries() {
//         if entry.type_ == MEMMAP_USABLE {
//             if entry.length > largest_size {
//                 largest_base = entry.base;
//                 largest_size = entry.length;
//             }
//         }
//     }

//     assert!(largest_size > 0, "Nenhuma regiao de memoria usavel encontrada para o heap");

//     // Aqui evitamos que o Heap engula a RAM inteira do sistema
//     let heap_size_to_use = if largest_size as usize > KERNEL_HEAP_SIZE_LIMIT {
//         KERNEL_HEAP_SIZE_LIMIT
//     } else {
//         largest_size as usize
//     };

//     let virtual_heap = largest_base + hhdm_offset; 
//     unsafe {
//         ALLOCATOR.lock().init(virtual_heap as *mut u8, heap_size_to_use);
//     }

//     // Retorna a base física e o tamanho exato que o heap REALMENTE se apropriou
//     (largest_base as usize, heap_size_to_use)
// }

pub fn init() -> (usize, usize) {
    let hhdm_offset = HHDM_REQUEST
        .response()
        .map(|r| r.offset)
        .expect("HHDM request failed - bootloader nao respondeu");

    let mut largest_base = 0;
    let mut largest_size = 0;

    if let Some(response) = MEMORY_MAP_REQUEST.response() {
        for entry in response.entries() {
            if entry.type_ == MEMMAP_USABLE {
                if entry.length > largest_size {
                    largest_base = entry.base;
                    largest_size = entry.length;
                }
            }
        }

        if largest_size > 0 {
            // SEGURANÇA: Limitar o Heap a 32 MB para não causar falhas nos limites da RAM!
            let heap_size = if largest_size > (32 * 1024 * 1024) {
                32 * 1024 * 1024
            } else {
                largest_size as usize
            };

            let virtual_heap = largest_base + hhdm_offset; 
            unsafe {
                ALLOCATOR.lock().init(virtual_heap as *mut u8, heap_size);
            }
            return (largest_base as usize, heap_size);
        }
    }
    
    (0, 0) 
}