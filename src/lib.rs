mod pnml;
mod pt_net_package;
mod pxml;
mod tests;
mod trait_impls;

use crate::pnml::{ObjectBase, PNMLName, PNMLVersion, PNMLID};

// pnml standard: http://cs.au.dk/fileadmin/site_files/cs/research_areas/centers_and_projects/cpn/paper06.pdf

///
/// Implementation of parts of the pnml core model and the PT-Net extension to use it as input for petri net model checkers.
///
///
///
///### Unsupported:
/// - graphics information for objects
/// - page (or global) labels
/// - tool specific information
#[derive(Debug)]
pub struct PNMLDocument {
    petri_nets: Vec<PetriNet>,
}

pub type Result<T> = std::result::Result<T, PetriError>;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct PetriNetRef(usize);

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PageRef {
    net: PNMLID,
    /// The path to the (sub) page as stack of indices
    /// The first element of the stack is the index of petri net pages
    /// The n+1th is the index of the sub page in the nth page
    page_stack: Vec<usize>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum NodeRef {
    PlaceRef { page: PageRef, obj_index: usize },
    TransitionRef { page: PageRef, obj_index: usize },
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ArcRef {
    page: PageRef,
    obj_index: usize,
    source: NodeRef,
    sink: NodeRef,
}

#[derive(Debug)]
pub struct PetriNet {
    id: PNMLID,
    typ: PNMLVersion,
    name: PNMLName,
    pages: Vec<ObjectBase>,
}

pub enum PetriError {
    BipartitionViolation,
    PlaceNotFound,
    TransitionNotFound,
    PageNotFound,
    ObjectNotFound,
    InvalidData(String),
    CorruptedData(String),
    XmlWriterError(xml::writer::Error),
}