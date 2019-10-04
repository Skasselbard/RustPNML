use crate::{
    pnml::{Label, Node, Object, ObjectBase, Page},
    PNMLDocument, PNMLName, PetriNet, Result,
};
use xml;
use xml::writer::{EmitterConfig, XmlEvent};

impl PNMLDocument {
    pub fn to_xml(&self) -> Result<String> {
        let mut writer = Vec::new();
        let mut xml_writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut writer);
        self.write_xml(&mut xml_writer)?;
        Ok(String::from_utf8(writer).expect("Document generated non UTF-8 string"))
    }
}

trait XmlAble<T>
where
    T: std::io::Write,
{
    fn write_xml(&self, writer: &mut xml::writer::EventWriter<T>) -> Result<()>;
}

impl<T> XmlAble<T> for PNMLDocument
where
    T: std::io::Write,
{
    fn write_xml(&self, writer: &mut xml::writer::EventWriter<T>) -> Result<()> {
        writer.write(
            XmlEvent::start_element("pnml")
                .default_ns("http://www.pnml.org/version-2009/grammar/pnml"),
        )?;
        for net in &self.petri_nets {
            net.write_xml(writer)?;
        }
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

impl<T> XmlAble<T> for PNMLName
where
    T: std::io::Write,
{
    fn write_xml(&self, writer: &mut xml::writer::EventWriter<T>) -> Result<()> {
        if let PNMLName(Some(name)) = &self {
            writer.write(XmlEvent::start_element("name"))?;
            writer.write(XmlEvent::start_element("text"))?;
            writer.write(XmlEvent::Characters(name))?;
            writer.write(XmlEvent::end_element())?;
            writer.write(XmlEvent::end_element())?;
        };
        Ok(())
    }
}

impl<T> XmlAble<T> for PetriNet
where
    T: std::io::Write,
{
    fn write_xml(&self, writer: &mut xml::writer::EventWriter<T>) -> Result<()> {
        writer.write(
            XmlEvent::start_element("net")
                .attr("id", &self.id.0)
                .attr("type", &self.typ.to_string()),
        )?;
        self.name.write_xml(writer)?;
        for page in &self.pages {
            page.write_xml(writer)?;
        }
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

impl<T> XmlAble<T> for Page
where
    T: std::io::Write,
{
    fn write_xml(&self, writer: &mut xml::writer::EventWriter<T>) -> Result<()> {
        for object in &self.objects {
            object.write_xml(writer)?;
        }
        for page in &self.sub_pages {
            page.write_xml(writer)?;
        }
        Ok(())
    }
}

impl<T> XmlAble<T> for ObjectBase
where
    T: std::io::Write,
{
    fn write_xml(&self, writer: &mut xml::writer::EventWriter<T>) -> Result<()> {
        // get the correct start element label
        let start_element = match &self.object {
            Object::Arc(source, target) => XmlEvent::start_element("arc")
                .attr("source", &source.0)
                .attr("target", &target.0),
            Object::Node(node) => node.start_element(),
            Object::Page(page) => page.start_element(),
        };
        let start_element = start_element.attr("id", &self.id.0);
        // write the start element
        writer.write(start_element)?;
        {
            // write the contained tags
            self.name.write_xml(writer)?;
            if let Some(labels) = &self.labels {
                for label in labels {
                    label.write_xml(writer)?;
                }
            }
            // write page content if its a page
            match &self.object {
                Object::Page(page) => page.write_xml(writer)?,
                _ => {}
            }
        } // end of internal tags -> write the end element
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

impl<T> XmlAble<T> for Label
where
    T: std::io::Write,
{
    fn write_xml(&self, writer: &mut xml::writer::EventWriter<T>) -> Result<()> {
        match self {
            Label::PTAnnotation(weight) => {
                writer.write(XmlEvent::start_element("inscription"))?;
                writer.write(XmlEvent::start_element("text"))?;
                writer.write(XmlEvent::Characters(&weight.to_string()))?;
            }
            Label::PTMarking(mark_count) => {
                writer.write(XmlEvent::start_element("initialMarking"))?;
                writer.write(XmlEvent::start_element("text"))?;
                writer.write(XmlEvent::Characters(&mark_count.to_string()))?;
            }
        };
        writer.write(XmlEvent::end_element())?;
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

trait XmlElementAble {
    fn start_element(&self) -> xml::writer::events::StartElementBuilder<'_>;
}

impl XmlElementAble for Page {
    fn start_element(&self) -> xml::writer::events::StartElementBuilder<'_> {
        XmlEvent::start_element("page")
    }
}

impl XmlElementAble for Node {
    fn start_element(&self) -> xml::writer::events::StartElementBuilder<'_> {
        match self {
            Node::Place => XmlEvent::start_element("place"),
            Node::Transition => XmlEvent::start_element("transition"),
            Node::PlaceRef(id, _) => XmlEvent::start_element("referencePlace").attr("ref", &id.0),
            Node::TransitionRef(id, _) => {
                XmlEvent::start_element("referenceTransition").attr("ref", &id.0)
            }
        }
    }
}
