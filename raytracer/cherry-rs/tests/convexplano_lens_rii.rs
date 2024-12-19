use std::rc::Rc;

mod common;

#[cfg(feature = "ri-info")]
#[test]
fn test_lens() {
    use common::ri_info::load_store;

    let mut store = load_store().unwrap();

    // Remove the item so that we can pass ownership to a Rc.
    let air = Rc::new(store.remove("other:air:Ciddor").unwrap());
    let nbk7 = Rc::new(store.remove("glass:BK7:SCHOTT").unwrap());
}
