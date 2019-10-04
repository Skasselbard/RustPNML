use crate::pnml::*;
use crate::*;

impl NodeRef {
    pub fn initial_marking(&mut self, net: &mut PetriNet, label: usize) -> Result<&mut Self> {
        const ERROR: &str = "transitions cannot have a marking";
        match self {
            NodeRef::PlaceRef { .. } => {
                let mut obj = net.get_node_obj_mut(self)?;
                match obj.labels {
                    None => {
                        obj.labels = Some(Vec::new());
                    }
                    _ => {}
                };
                let labels = obj.labels.as_mut().expect("this should not happen"); // we've just initialized you!
                labels.retain(|x| match x {
                    Label::PTMarking(_) => false,
                    _ => true,
                });
                labels.push(Label::PTMarking(label));
                Ok(self)
            }
            NodeRef::TransitionRef { .. } => Err(PetriError::InvalidData(String::from(ERROR))),
        }
    }
}

impl ArcRef {
    /// sets the weight/multiplicity of the arc
    pub fn inscription(
        &mut self,
        net: &mut PetriNet,
        label: std::num::NonZeroUsize,
    ) -> Result<&mut Self> {
        let mut obj = net.get_arc_obj_mut(self)?;
        match obj.labels {
            None => {
                obj.labels = Some(Vec::new());
            }
            _ => {}
        };
        let labels = obj.labels.as_mut().expect("this should not happen"); // we've just initialized you!
        labels.retain(|x| match x {
            Label::PTAnnotation(_) => false,
            _ => true,
        });
        labels.push(Label::PTAnnotation(label));
        Ok(self)
    }
}
