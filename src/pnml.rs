use crate::*;
use std;

#[derive(Debug, Clone)]
pub(crate) struct PNMLName(pub(crate) Option<String>);

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct PNMLID(pub(crate) String);

#[derive(Debug)]
pub(crate) enum PNMLVersion {
    V2009,
}

#[derive(Debug)]
pub(crate) struct Page {
    pub(crate) objects: Vec<ObjectBase>,
    pub(crate) sub_pages: Vec<ObjectBase>,
}

#[derive(Debug)]
pub(crate) struct ObjectBase {
    pub(crate) id: PNMLID,
    pub(crate) name: PNMLName,
    pub(crate) labels: Option<Vec<Label>>,
    pub(crate) object: Object,
}

#[derive(Debug)]
pub(crate) enum Object {
    Page(Page),
    Arc(PNMLID, PNMLID),
    Node(Node),
}

#[derive(Debug)]
pub(crate) enum Node {
    Place,
    Transition,
    PlaceRef(PNMLID, NodeRef),
    TransitionRef(PNMLID, NodeRef),
}

#[derive(Debug)]
pub(crate) enum Label {
    /// Mark count on a Place
    PTMarking(usize),
    /// Multiplicity of an arc
    /// cannot be zero by definition
    PTAnnotation(std::num::NonZeroUsize),
}

impl PNMLName {
    pub fn new(name: &str) -> Self {
        PNMLName(Some(name.into()))
    }
}

impl PNMLID {
    pub fn new(id: &str) -> Self {
        PNMLID(id.into())
    }
}

impl PNMLDocument {
    pub fn new() -> Self {
        PNMLDocument {
            petri_nets: Vec::new(),
        }
    }

    pub fn add_petri_net(&mut self, name: Option<&str>) -> PetriNetRef {
        self.petri_nets.push(PetriNet {
            id: PNMLID::new(&format!("n{}", self.petri_nets.len())),
            typ: PNMLVersion::V2009,
            name: name.into(),
            pages: Vec::new(),
        });
        PetriNetRef(self.petri_nets.len() - 1)
    }

    pub fn petri_net_data(&mut self, net: PetriNetRef) -> Result<&mut PetriNet> {
        self.petri_nets.get_mut(net.0).ok_or(PetriError::NetNotFound)
    }

    pub fn petri_nets(&self) -> Vec<PetriNetRef> {
        self.petri_nets
            .iter()
            .enumerate()
            .map(|(i, _)| PetriNetRef(i))
            .collect()
    }
}

impl PetriNet {
    pub fn pages(&self) -> Vec<PageRef> {
        self.pages
            .iter()
            .enumerate()
            // though we iterate other ObjectBases the indices are consistent
            .map(|(i, _)| {
                let mut page_stack = Vec::new();
                page_stack.push(i);
                PageRef {
                    net: self.id.clone(),
                    page_stack,
                }
            })
            .collect()
    }

    pub fn add_page(&mut self, name: Option<&str>) -> PageRef {
        self.pages.push(ObjectBase {
            id: PNMLID::new(&format!("{}_p{}", self.id.0, self.pages.len())),
            name: name.into(),
            labels: None,
            object: Object::Page(Page {
                objects: Vec::new(),
                sub_pages: Vec::new(),
            }),
        });
        let mut page_stack = Vec::new();
        page_stack.push(self.pages.len() - 1);
        PageRef {
            net: self.id.clone(),
            page_stack,
        }
    }

    pub fn add_sub_page(&mut self, name: Option<&str>, parent: &PageRef) -> Result<PageRef> {
        let page = self.get_page_mut(parent)?;
        // remember the parent id to build a new one from it
        let parent_id = page.id.0.clone();
        // make the actual page out of the object
        let page = PetriNet::obj_to_page_mut(page)?;
        // finally add the page and return the reference
        page.sub_pages.push(ObjectBase {
            id: PNMLID::new(&format!("{}-{}", parent_id, page.sub_pages.len())),
            name: name.into(),
            labels: None,
            object: Object::Page(Page {
                objects: Vec::new(),
                sub_pages: Vec::new(),
            }),
        });
        let mut page_stack = parent.page_stack.clone();
        page_stack.push(self.pages.len() - 1);
        Ok(PageRef {
            net: self.id.clone(),
            page_stack,
        })
    }

    pub fn add_place(&mut self, page_ref: &PageRef) -> Result<NodeRef> {
        let page = self.get_page_mut(page_ref)?;
        let parent_id = page.id.0.clone();
        let page = PetriNet::obj_to_page_mut(page)?;
        page.objects.push(ObjectBase {
            id: PNMLID::new(&format!("{}_o{}", parent_id, page.objects.len())),
            name: None.into(),
            labels: None,
            object: Object::Node(Node::Place),
        });
        Ok(NodeRef::PlaceRef {
            page: page_ref.clone(),
            obj_index: page.objects.len() - 1,
        })
    }

    pub fn add_transition(&mut self, page_ref: &PageRef) -> Result<NodeRef> {
        let page = self.get_page_mut(page_ref)?;
        let parent_id = page.id.0.clone();
        let page = PetriNet::obj_to_page_mut(page)?;
        page.objects.push(ObjectBase {
            id: PNMLID::new(&format!("{}_o{}", parent_id, page.objects.len())),
            name: None.into(),
            labels: None,
            object: Object::Node(Node::Transition),
        });
        Ok(NodeRef::TransitionRef {
            page: page_ref.clone(),
            obj_index: page.objects.len() - 1,
        })
    }

    pub fn add_arc(
        &mut self,
        page_ref: &PageRef,
        source_ref: &NodeRef,
        sink_ref: &NodeRef,
    ) -> Result<ArcRef> {
        // make sure places and transitions are connected correctly
        match (&source_ref, &sink_ref) {
            (NodeRef::PlaceRef { .. }, NodeRef::PlaceRef { .. }) => {
                return Err(PetriError::BipartitionViolation)
            }
            (NodeRef::TransitionRef { .. }, NodeRef::TransitionRef { .. }) => {
                return Err(PetriError::BipartitionViolation)
            }
            _ => {}
        };
        let source_id = self.get_node_obj_mut(source_ref)?.id.clone();
        let sink_id = self.get_node_obj_mut(sink_ref)?.id.clone();
        let page = self.get_page_mut(page_ref)?;
        let parent_id = page.id.0.clone();
        let page = PetriNet::obj_to_page_mut(page)?;
        page.objects.push(ObjectBase {
            id: PNMLID::new(&format!("{}_o{}", parent_id, page.objects.len())),
            name: None.into(),
            labels: None,
            object: Object::Arc(source_id, sink_id),
        });
        Ok(ArcRef {
            page: page_ref.clone(),
            obj_index: page.objects.len() - 1,
            source: source_ref.clone(),
            sink: sink_ref.clone(),
        })
    }

    /// Creates a PNML standard RefPlace or RefTrans respectively.
    ///
    /// Unlike the NodeRef::* enum variants that are used to look up a position in the
    /// internal data structure, a RefPlace/RefTrans is an object in the pnml standard,
    /// that is used as a link to a Place/Transition in another location (e.g. another
    /// page). The RefPlace/RefTrans represent the same Place/Transition they link to.
    /// Such a reference is helpful to mention the same Node on different pages (in the
    /// graphical representation)
    pub fn add_reference_node(&mut self, reference: &NodeRef, page: &PageRef) -> Result<()> {
        let ref_obj = self.get_node_obj(reference)?;
        let ref_id = ref_obj.id.clone();
        let reference_node = match reference {
            NodeRef::PlaceRef { .. } => Node::PlaceRef(ref_id, reference.clone()),
            NodeRef::TransitionRef { .. } => Node::TransitionRef(ref_id, reference.clone()),
        };
        let ref_id = ref_obj.id.0.clone();
        let page = PetriNet::obj_to_page_mut(self.get_page_mut(page)?)?;
        page.objects.push(ObjectBase {
            id: PNMLID::new(&format!("{}_ref_o{}", ref_id, page.objects.len())),
            name: None.into(),
            labels: None,
            object: Object::Node(reference_node),
        });
        Ok(())
    }

    pub(crate) fn obj_to_page_mut(o: &mut ObjectBase) -> Result<&mut Page> {
        const WRONG_OBJECT_MSG: &str = "Object in the sub pages array which is no Page";
        match &mut o.object {
            Object::Page(page) => Ok(page),
            _ => return Err(PetriError::CorruptedData(WRONG_OBJECT_MSG.into())),
        }
    }

    pub(crate) fn obj_to_page(o: &ObjectBase) -> Result<&Page> {
        const WRONG_OBJECT_MSG: &str = "Object in the sub pages array which is no Page";
        match &o.object {
            Object::Page(page) => Ok(page),
            _ => return Err(PetriError::CorruptedData(WRONG_OBJECT_MSG.into())),
        }
    }

    /// searches the referenced page in the net
    /// returns an ObjectBase because there is additional information stored (like the PNMLID)
    pub(crate) fn get_page_mut(&mut self, page: &PageRef) -> Result<&mut ObjectBase> {
        const EMPTY_STACK_MSG: &str = "Invalid PageRef: Stack was empty";
        /// get the sub page with index i from page_base
        fn get_sub_page(page_base: &mut ObjectBase, i: usize) -> Result<&mut ObjectBase> {
            PetriNet::obj_to_page_mut(page_base)?
                .sub_pages
                .get_mut(i)
                .ok_or(PetriError::PageNotFound)
        }
        // clone the index array to mess around safely
        let mut stack = page.page_stack.clone();
        // get the initial page that is no sub page (stored directly in a PetriNet)
        // thank you auto formatter :D
        let mut page = self
            .pages
            .get_mut(
                stack
                    .pop()
                    .ok_or(PetriError::CorruptedData(EMPTY_STACK_MSG.into()))?,
            )
            .ok_or(PetriError::PageNotFound)?;
        // traverse the sub and subsub pages
        for index in stack {
            page = get_sub_page(page, index)?;
        }
        Ok(page)
    }

    pub(crate) fn get_page(&self, page: &PageRef) -> Result<&ObjectBase> {
        const EMPTY_STACK_MSG: &str = "Invalid PageRef: Stack was empty";
        /// get the sub page with index i from page_base
        fn get_sub_page(page_base: &ObjectBase, i: usize) -> Result<&ObjectBase> {
            PetriNet::obj_to_page(page_base)?
                .sub_pages
                .get(i)
                .ok_or(PetriError::PageNotFound)
        }
        // clone the index array to mess around safely
        let mut stack = page.page_stack.clone();
        // get the initial page that is no sub page (stored directly in a PetriNet)
        // thank you auto formatter :D
        let mut page = self
            .pages
            .get(
                stack
                    .pop()
                    .ok_or(PetriError::CorruptedData(EMPTY_STACK_MSG.into()))?,
            )
            .ok_or(PetriError::PageNotFound)?;
        // traverse the sub and subsub pages
        for index in stack {
            page = get_sub_page(page, index)?;
        }
        Ok(page)
    }

    pub(crate) fn get_node_obj_mut(&mut self, node: &NodeRef) -> Result<&mut ObjectBase> {
        let (page_ref, obj_index) = match node {
            NodeRef::PlaceRef { page, obj_index } => (page, obj_index),
            NodeRef::TransitionRef { page, obj_index } => (page, obj_index),
        };
        let page = PetriNet::obj_to_page_mut(self.get_page_mut(&page_ref)?)?;
        Ok(page
            .objects
            .get_mut(*obj_index)
            .ok_or(PetriError::ObjectNotFound)?)
    }

    pub(crate) fn get_node_obj(&self, node: &NodeRef) -> Result<&ObjectBase> {
        let (page_ref, obj_index) = match node {
            NodeRef::PlaceRef { page, obj_index } => (page, obj_index),
            NodeRef::TransitionRef { page, obj_index } => (page, obj_index),
        };
        let page = PetriNet::obj_to_page(self.get_page(&page_ref)?)?;
        Ok(page
            .objects
            .get(*obj_index)
            .ok_or(PetriError::ObjectNotFound)?)
    }

    pub(crate) fn get_arc_obj_mut(&mut self, arc: &ArcRef) -> Result<&mut ObjectBase> {
        let page = PetriNet::obj_to_page_mut(self.get_page_mut(&arc.page)?)?;
        let arc = page
            .objects
            .get_mut(arc.obj_index)
            .ok_or(PetriError::ObjectNotFound)?;
        assert!(match arc.object {
            Object::Arc { .. } => true,
            _ => false,
        });
        Ok(arc)
    }

    pub(crate) fn get_arc_obj(&self, arc: &ArcRef) -> Result<&ObjectBase> {
        let page = PetriNet::obj_to_page(self.get_page(&arc.page)?)?;
        let arc = page
            .objects
            .get(arc.obj_index)
            .ok_or(PetriError::ObjectNotFound)?;
        assert!(match arc.object {
            Object::Arc { .. } => true,
            _ => false,
        });
        Ok(arc)
    }
}

impl NodeRef {
    pub fn name(&mut self, net: &mut PetriNet, name: &str) -> Result<(&mut Self)> {
        let mut obj = net.get_node_obj_mut(self)?;
        obj.name = PNMLName::new(name);
        Ok(self)
    }

    pub fn get_name<'a>(&'a self, net: &'a PetriNet) -> Result<Option<&'a str>> {
        let obj = net.get_node_obj(self)?;
        match &obj.name {
            PNMLName(Some(name)) => Ok(Some(&name)),
            PNMLName(None) => Ok(None),
        }
    }
}

impl ArcRef {
    pub fn name(&mut self, net: &mut PetriNet, name: &str) -> Result<&mut Self> {
        let mut obj = net.get_arc_obj_mut(self)?;
        obj.name = PNMLName::new(name);
        Ok(self)
    }

    pub fn get_name<'a>(&'a self, net: &'a PetriNet) -> Result<Option<&'a str>> {
        let obj = net.get_arc_obj(self)?;
        match &obj.name {
            PNMLName(Some(name)) => Ok(Some(&name)),
            PNMLName(None) => Ok(None),
        }
    }
}
