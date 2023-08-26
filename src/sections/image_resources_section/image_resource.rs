use crate::sections::image_resources_section::DescriptorStructure;
use crate::sections::image_resources_section::ImageResourcesBlock;

/// An image resource from the image resources section
#[derive(Debug)]
#[allow(missing_docs)]
pub enum ImageResource {
    Slices(SlicesImageResource),
    Timeline(TimelineImageResource),
    Xmp(String),
}

#[derive(Debug)]
pub struct TimelineImageResource {
    pub(crate) block: ImageResourcesBlock,
}

/// Comes from a slices resource block
#[derive(Debug)]
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
