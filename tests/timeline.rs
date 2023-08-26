use psd::Psd;

/// Verify that we can parse timeline information
#[test]
fn parse_timeline() {
    println!("./fixtures/timeline/2017-frame-timeline-1frame-smiley.psd");
    let psd = include_bytes!("./fixtures/timeline/2017-frame-timeline-1frame-smiley.psd");
    Psd::from_bytes(psd).unwrap();

    println!("./fixtures/timeline/2017-frame-timeline-4frame-smiley.psd");
    let psd = include_bytes!("./fixtures/timeline/2017-frame-timeline-4frame-smiley.psd");
    Psd::from_bytes(psd).unwrap();

    println!("./fixtures/timeline/2017-frame-timeline-4frame-smiley-long-2nd.psd");
    let psd = include_bytes!("./fixtures/timeline/2017-frame-timeline-4frame-smiley-long-2nd.psd");
    Psd::from_bytes(psd).unwrap();

    println!("./fixtures/timeline/2017-frame-timeline-5frame-smiley-3-selected.psd");
    let psd = include_bytes!("./fixtures/timeline/2017-frame-timeline-5frame-smiley-3-selected.psd");
    Psd::from_bytes(psd).unwrap();
}
