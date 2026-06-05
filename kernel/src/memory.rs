use limine::request::{MemmapRequest, HhdmRequest};
use limine::memmap::MEMMAP_USABLE;
use linked_list_allocator::LockedHeap;


#[used]
#[link_section = ".requests"]
static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new();


#[used]
#[link_section = ".requests"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();



#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
    let HhdmOffset = HHDM_REQUEST
        .response()
        .map(|r| r.offset)
        .unwrap_or(0);

    if let Some(response) = MEMORY_MAP_REQUEST.response() {
        let mut largest_base = 0;
        let mut largest_size = 0;

        for entry in response.entries() {
            if entry.type_ == MEMMAP_USABLE {
                if entry.length > largest_size {
                    largest_base = entry.base;
                    largest_size = entry.length;
                }
            }
        }


        if largest_size > 0 {
            let vitual_heap = largest_base + HhdmOffset; 
            unsafe {
                ALLOCATOR.lock().init(vitual_heap as *mut u8, largest_size as usize);
            }
        }
    }
}
