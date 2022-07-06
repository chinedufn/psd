use crate::sections::image_resources_section::DescriptorStructure;
use serde::Serialize;

/// An image resource from the image resources section
#[derive(Debug, Serialize)]
#[allow(missing_docs)]
pub enum ImageResource {
    Slices(SlicesImageResource),
}

/// Comes from a slices resource block
#[derive(Debug, Serialize)]
pub struct SlicesImageResource {
    pub(crate) name: String,
    pub(crate) descriptors: Vec<DescriptorStructure>,
}

#[allow(missing_docs)]
impl SlicesImageResource {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn descriptors(&self) -> &Vec<DescriptorStructure> {
        &self.descriptors
    }
}
