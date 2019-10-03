#[test]
fn serialize() {
    use crate::pnml::*;
    use crate::*;
    let mut doc = PNMLDocument::new();
    let net_ref1 = doc.add_petri_net(Some("net1"));
    let net1 = doc.petri_net_data(net_ref1).unwrap();
    let page_ref1 = net1.add_page(Some("page1"));
    let sub_page_ref1 = net1.add_sub_page(Some("subPage"), &page_ref1).unwrap();
    // let place1 = net1.add_place(Some("p1"), &page_ref1).unwrap();
    // let transition1 = net1.add_transition(Some("t1"), &sub_page_ref1).unwrap();
    // let place2 = net1.add_place(Some("p2"), &sub_page_ref1).unwrap();
    // let place3 = net1.add_place(Some("p3"), &sub_page_ref1).unwrap();
    // let place4 = net1.add_place(Some("p4"), &page_ref1).unwrap();
    // let arc = net1.add_arc(Some("a1"), &page_ref1, &place1, &transition1);
    println!("{}", doc.to_xml().unwrap());
}
