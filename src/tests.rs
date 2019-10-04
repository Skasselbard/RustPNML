#[test]
fn serialize() {
    use crate::*;
    use std::num::NonZeroUsize;
    let mut doc = PNMLDocument::new();
    let net_ref1 = doc.add_petri_net(Some("net1"));
    let net1 = doc.petri_net_data(net_ref1).unwrap();
    let page_ref1 = net1.add_page(Some("page1"));
    let sub_page_ref1 = net1.add_sub_page(Some("subPage"), &page_ref1).unwrap();
    let place1 = net1.add_place(&page_ref1).unwrap();
    let transition1 = net1.add_transition(&sub_page_ref1).unwrap();
    let mut place2 = net1.add_place(&sub_page_ref1).unwrap();
    let mut place3 = net1.add_place(&sub_page_ref1).unwrap();
    let place4 = net1.add_place(&page_ref1).unwrap();
    let mut arc = net1.add_arc(&page_ref1, &place1, &transition1).unwrap();
    place2.name(net1, "p2").unwrap();
    arc.inscription(net1, NonZeroUsize::new(5usize).unwrap())
        .unwrap();
    place3.initial_marking(net1, 7).unwrap();
    println!("{}", doc.to_xml().unwrap());
}
