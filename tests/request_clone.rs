//! Request copies in preparation must not allocate a new decoded field tree.
use forever_engine::contracts::request::{Message, Request};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct CountingAllocator;

thread_local! {
    static ALLOCATIONS: Cell<Option<usize>> = const { Cell::new(None) };
}

fn count_allocation() {
    let _ = ALLOCATIONS.try_with(|count| {
        if let Some(value) = count.get() {
            count.set(Some(value + 1));
        }
    });
}

// SAFETY: Every operation forwards the original pointer and layout to System.
// The counter uses allocation-free thread-local storage and does not touch allocations.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count_allocation();
        // SAFETY: The caller supplies the layout required by GlobalAlloc.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: The allocation came from System with this same layout.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count_allocation();
        // SAFETY: The caller supplies the layout required by GlobalAlloc.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count_allocation();
        // SAFETY: The caller supplies a System allocation and valid replacement size.
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[test]
fn cloning_a_decoded_request_does_not_allocate() {
    let request =
        Request::from_json(br#"{"simOptions":{"iterations":3000,"randomSeed":"7","debug":true}}"#)
            .unwrap();
    ALLOCATIONS.with(|count| count.set(Some(0)));
    let cloned = std::hint::black_box(&request).clone();
    let allocations = ALLOCATIONS.with(|count| count.replace(None).unwrap());
    assert_eq!(allocations, 0, "request clones should share decoded data");
    assert_eq!(cloned.encode(), request.encode());
    assert_eq!(cloned.sha256(), request.sha256());
}

#[test]
fn cloned_messages_keep_independent_scalar_values() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Request>();
    assert_send_sync::<Message>();

    let original =
        Message::from_json_text("proto.SimOptions", r#"{"iterations":3000,"debug":true}"#).unwrap();
    let encoded = original.encode();
    let mut cleared = original.clone();
    cleared.set_bool("debug", false);
    cleared.set_i32("iterations", 0);
    assert_eq!(cleared.to_protojson(), serde_json::json!({}));
    assert_eq!(cleared.encode(), Vec::<u8>::new());
    assert_eq!(original.encode(), encoded);
    assert_eq!(
        original.to_protojson(),
        serde_json::json!({"iterations":3000,"debug":true})
    );

    let mut changed = original.clone();
    changed.set_bool("debug", false);
    changed.set_i32("iterations", 7);
    assert_eq!(changed.to_protojson(), serde_json::json!({"iterations":7}));
    assert_eq!(original.encode(), encoded);

    let empty = Message::empty("proto.SimOptions");
    let mut populated = empty.clone();
    populated.set_bool("debug", true);
    populated.set_i32("iterations", 2);
    assert_eq!(
        populated.to_protojson(),
        serde_json::json!({"iterations":2,"debug":true})
    );
    assert_eq!(empty.to_protojson(), serde_json::json!({}));
}

#[test]
fn changing_a_nested_message_clone_preserves_the_request() {
    let request = Request::from_json(br#"{"simOptions":{"iterations":3,"debug":true}}"#).unwrap();
    let encoded = request.encode();
    let digest = request.sha256();
    let mut options = request.message().message("sim_options").unwrap().clone();
    options.set_i32("iterations", 9);
    options.set_bool("debug", false);
    assert_eq!(options.to_protojson(), serde_json::json!({"iterations":9}));
    assert_eq!(request.encode(), encoded);
    assert_eq!(request.sha256(), digest);
    assert_eq!(
        request.message().to_protojson(),
        serde_json::json!({"simOptions":{"iterations":3,"debug":true}})
    );
}
