use psd::{DescriptorField, ImageResource, Psd};

/// In this test we check that root descriptor's `bounds` field is equal to 1
/// So, then fields parsed correctly
///
/// cargo test --test image_resources_section image_check_1x1p_bound_field -- --exact
#[test]
fn image_check_1x1p_bound_field() {
    let psd = include_bytes!("./fixtures/two-layers-red-green-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    let descriptors = match &psd.resources()[1] {
        ImageResource::Slices(s) => s.descriptors(),
        ImageResource::Xmp(..) => panic!("unexpected resource ordering"),
    };
    let descriptor = descriptors.get(0).unwrap();
    let bounds = descriptor.fields.get("bounds").unwrap();

    if let DescriptorField::Descriptor(d) = bounds {
        match d.fields.get("Rght").unwrap() {
            DescriptorField::Integer(v) => assert_eq!(*v, 1),
            _ => panic!("expected integer"),
        }

        match d.fields.get("Btom").unwrap() {
            DescriptorField::Integer(v) => assert_eq!(*v, 1),
            _ => panic!("expected integer"),
        }
    } else {
        panic!("expected descriptor");
    }
}

/// In this test we check that root descriptor's `bounds` field is equal to 16
/// So, then fields parsed correctly
///
/// cargo test --test image_resources_section image_check_16x16p_bound_field -- --exact
#[test]
fn image_check_16x16p_bound_field() {
    let psd = include_bytes!("./fixtures/16x16-rle-partially-opaque.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    let descriptors = match &psd.resources()[1] {
        ImageResource::Slices(s) => s.descriptors(),
        ImageResource::Xmp(..) => panic!("unexpected resource ordering"),
    };
    let descriptor = descriptors.get(0).unwrap();
    let bounds = descriptor.fields.get("bounds").unwrap();

    if let DescriptorField::Descriptor(d) = bounds {
        match d.fields.get("Rght").unwrap() {
            DescriptorField::Integer(v) => assert_eq!(*v, 16),
            _ => panic!("expected integer"),
        }

        match d.fields.get("Btom").unwrap() {
            DescriptorField::Integer(v) => assert_eq!(*v, 16),
            _ => panic!("expected integer"),
        }
    } else {
        panic!("expected descriptor");
    }
}

/// The image contains a non-UTF-8 Pascal string of even length in its image resource block.
///
/// cargo test --test image_resources_section image_non_utf8_pascal_string -- --exact
#[test]
fn image_non_utf8_pascal_string() {
    let psd = include_bytes!("./fixtures/non-utf8-pascal-string.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert!(psd.layers().is_empty());
}

/// The image contains a Pascal string of odd length in its image resource block.
///
/// cargo test --test image_resources_section image_odd_length_pascal_string -- --exact
#[test]
fn image_odd_length_pascal_string() {
    let psd = include_bytes!("./fixtures/odd-length-pascal-string.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert!(psd.layers().is_empty());
}

/// Check that the XMP data is parsed correctly from one of the fixtures.
///
/// cargo test --test image_resources_section xmp_check_parsed_as_string -- --exact
#[test]
fn xmp_check_parsed_as_string() {
    let psd = include_bytes!("./fixtures/two-layers-red-green-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    let xmp_string = match &psd.resources()[0] {
        ImageResource::Xmp(s) => s,
        ImageResource::Slices(..) => panic!("unexpected resource ordering"),
    };
    assert_eq!(xmp_string, "<?xpacket begin=\"\u{feff}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>
<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"Adobe XMP Core 5.6-c140 79.160451, 2017/05/06-01:08:21        \">
   <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">
      <rdf:Description rdf:about=\"\"
            xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\"
            xmlns:dc=\"http://purl.org/dc/elements/1.1/\"
            xmlns:xmpMM=\"http://ns.adobe.com/xap/1.0/mm/\"
            xmlns:stEvt=\"http://ns.adobe.com/xap/1.0/sType/ResourceEvent#\"
            xmlns:photoshop=\"http://ns.adobe.com/photoshop/1.0/\">
         <xmp:CreatorTool>Adobe Photoshop CC 2018 (Macintosh)</xmp:CreatorTool>
         <xmp:CreateDate>2019-02-18T21:23:30-05:00</xmp:CreateDate>
         <xmp:MetadataDate>2019-02-18T21:23:41-05:00</xmp:MetadataDate>
         <xmp:ModifyDate>2019-02-18T21:23:41-05:00</xmp:ModifyDate>
         <dc:format>application/vnd.adobe.photoshop</dc:format>
         <xmpMM:InstanceID>xmp.iid:f88d4ee4-9872-4982-a8b8-a0ed4e5deb99</xmpMM:InstanceID>
         <xmpMM:DocumentID>xmp.did:031c6a78-d72a-48c5-926c-bcc201d2b60b</xmpMM:DocumentID>
         <xmpMM:OriginalDocumentID>xmp.did:031c6a78-d72a-48c5-926c-bcc201d2b60b</xmpMM:OriginalDocumentID>
         <xmpMM:History>
            <rdf:Seq>
               <rdf:li rdf:parseType=\"Resource\">
                  <stEvt:action>created</stEvt:action>
                  <stEvt:instanceID>xmp.iid:031c6a78-d72a-48c5-926c-bcc201d2b60b</stEvt:instanceID>
                  <stEvt:when>2019-02-18T21:23:30-05:00</stEvt:when>
                  <stEvt:softwareAgent>Adobe Photoshop CC 2018 (Macintosh)</stEvt:softwareAgent>
               </rdf:li>
               <rdf:li rdf:parseType=\"Resource\">
                  <stEvt:action>saved</stEvt:action>
                  <stEvt:instanceID>xmp.iid:f88d4ee4-9872-4982-a8b8-a0ed4e5deb99</stEvt:instanceID>
                  <stEvt:when>2019-02-18T21:23:41-05:00</stEvt:when>
                  <stEvt:softwareAgent>Adobe Photoshop CC 2018 (Macintosh)</stEvt:softwareAgent>
                  <stEvt:changed>/</stEvt:changed>
               </rdf:li>
            </rdf:Seq>
         </xmpMM:History>
         <photoshop:ColorMode>3</photoshop:ColorMode>
         <photoshop:ICCProfile>Display</photoshop:ICCProfile>
      </rdf:Description>
   </rdf:RDF>
</x:xmpmeta>
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                                                                                                    
                            
<?xpacket end=\"w\"?>");
}
