var searchIndex={};
searchIndex["psd"] = {"doc":"Data structures and methods for working with PSD files.","i":[[3,"PsdLayer","psd","Information about a layer in a PSD file.",null,null],[3,"Psd","","Represents the contents of a PSD file",null,null],[4,"PsdChannelCompression","","Indicates how a channe'sl data is compressed",null,null],[13,"RawData","","Not compressed",0,null],[13,"RleCompressed","","Compressed using PackBits RLE compression",0,null],[13,"ZipWithoutPrediction","","Currently unsupported",0,null],[13,"ZipWithPrediction","","Currently unsupported",0,null],[4,"PsdChannelKind","","The different kinds of channels in a layer (red, green,…",null,null],[13,"Red","","",1,null],[13,"Green","","",1,null],[13,"Blue","","",1,null],[13,"TransparencyMask","","",1,null],[13,"UserSuppliedLayerMask","","",1,null],[13,"RealUserSuppliedLayerMask","","",1,null],[4,"ColorMode","","Adobe Docs",null,null],[13,"Bitmap","","",2,null],[13,"Grayscale","","",2,null],[13,"Indexed","","",2,null],[13,"Rgb","","",2,null],[13,"Cmyk","","",2,null],[13,"Multichannel","","",2,null],[13,"Duotone","","",2,null],[13,"Lab","","",2,null],[4,"PsdDepth","","Adobe Docs",null,null],[13,"One","","",3,null],[13,"Eight","","",3,null],[13,"Sixteen","","",3,null],[13,"ThirtyTwo","","",3,null],[11,"new","","Create a new PsdLayerChannelCompression",0,[[],[["error",3],["psdchannelcompression",4],["result",4]]]],[11,"new","","Create a new PsdLayerChannel",1,[[],[["result",4],["psdchannelkind",4],["error",3]]]],[11,"rgba_offset","","R -> 0 G -> 1 B -> 2 A -> 3",1,[[],[["result",4],["error",3]]]],[11,"new","","Create a new PsdDepth",3,[[],[["error",3],["psddepth",4],["result",4]]]],[11,"new","","Create a new ColorMode",2,[[],[["colormode",4],["error",3],["result",4]]]],[11,"new","","Create a new photoshop layer",4,[[["string",3]],["psdlayer",3]]],[11,"name","","Get the name of the layer",4,[[]]],[11,"width","","The width of the layer",4,[[]]],[11,"height","","The height of the layer",4,[[]]],[11,"compression","","Get the compression level for one of this layer's channels",4,[[["psdchannelkind",4]],[["error",3],["psdchannelcompression",4],["result",4]]]],[11,"rgba","","Create a vector that interleaves the red, green, blue and…",4,[[],[["vec",3],["error",3],["result",4]]]],[11,"from_bytes","","Create a Psd from a byte slice.",5,[[],[["error",3],["psd",3],["result",4]]]],[11,"width","","The width of the PSD file",5,[[]]],[11,"height","","The height of the PSD file",5,[[]]],[11,"depth","","The number of bits per channel",5,[[],["psddepth",4]]],[11,"color_mode","","The color mode of the file",5,[[],["colormode",4]]],[11,"layers","","Get all of the layers in the PSD",5,[[],["vec",3]]],[11,"layer_by_name","","Get a layer by name",5,[[],[["error",3],["result",4],["psdlayer",3]]]],[11,"layer_by_idx","","Get a layer by index.",5,[[],[["error",3],["result",4],["psdlayer",3]]]],[11,"flatten_layers_rgba","","Given a filter, combine all layers in the PSD that pass…",5,[[["fn",8]],[["vec",3],["error",3],["result",4]]]],[11,"rgba","","Get the RGBA pixels for the PSD [ R,G,B,A, R,G,B,A,…",5,[[],["vec",3]]],[11,"compression","","Get the compression level for the flattened image data",5,[[],["psdchannelcompression",4]]],[11,"from","","",4,[[]]],[11,"into","","",4,[[]]],[11,"try_from","","",4,[[],["result",4]]],[11,"try_into","","",4,[[],["result",4]]],[11,"borrow","","",4,[[]]],[11,"borrow_mut","","",4,[[]]],[11,"type_id","","",4,[[],["typeid",3]]],[11,"from","","",5,[[]]],[11,"into","","",5,[[]]],[11,"try_from","","",5,[[],["result",4]]],[11,"try_into","","",5,[[],["result",4]]],[11,"borrow","","",5,[[]]],[11,"borrow_mut","","",5,[[]]],[11,"type_id","","",5,[[],["typeid",3]]],[11,"from","","",0,[[]]],[11,"into","","",0,[[]]],[11,"try_from","","",0,[[],["result",4]]],[11,"try_into","","",0,[[],["result",4]]],[11,"borrow","","",0,[[]]],[11,"borrow_mut","","",0,[[]]],[11,"type_id","","",0,[[],["typeid",3]]],[11,"from","","",1,[[]]],[11,"into","","",1,[[]]],[11,"to_owned","","",1,[[]]],[11,"clone_into","","",1,[[]]],[11,"try_from","","",1,[[],["result",4]]],[11,"try_into","","",1,[[],["result",4]]],[11,"borrow","","",1,[[]]],[11,"borrow_mut","","",1,[[]]],[11,"type_id","","",1,[[],["typeid",3]]],[11,"from","","",2,[[]]],[11,"into","","",2,[[]]],[11,"to_owned","","",2,[[]]],[11,"clone_into","","",2,[[]]],[11,"try_from","","",2,[[],["result",4]]],[11,"try_into","","",2,[[],["result",4]]],[11,"borrow","","",2,[[]]],[11,"borrow_mut","","",2,[[]]],[11,"type_id","","",2,[[],["typeid",3]]],[11,"from","","",3,[[]]],[11,"into","","",3,[[]]],[11,"to_owned","","",3,[[]]],[11,"clone_into","","",3,[[]]],[11,"try_from","","",3,[[],["result",4]]],[11,"try_into","","",3,[[],["result",4]]],[11,"borrow","","",3,[[]]],[11,"borrow_mut","","",3,[[]]],[11,"type_id","","",3,[[],["typeid",3]]],[11,"clone","","",1,[[],["psdchannelkind",4]]],[11,"clone","","",3,[[],["psddepth",4]]],[11,"clone","","",2,[[],["colormode",4]]],[11,"cmp","","",1,[[["psdchannelkind",4]],["ordering",4]]],[11,"eq","","",0,[[["psdchannelcompression",4]]]],[11,"eq","","",1,[[["psdchannelkind",4]]]],[11,"eq","","",3,[[["psddepth",4]]]],[11,"eq","","",2,[[["colormode",4]]]],[11,"partial_cmp","","",1,[[["psdchannelkind",4]],[["option",4],["ordering",4]]]],[11,"fmt","","",0,[[["formatter",3]],["result",6]]],[11,"fmt","","",1,[[["formatter",3]],["result",6]]],[11,"fmt","","",3,[[["formatter",3]],["result",6]]],[11,"fmt","","",2,[[["formatter",3]],["result",6]]],[11,"fmt","","",4,[[["formatter",3]],["result",6]]],[11,"fmt","","",5,[[["formatter",3]],["result",6]]],[11,"hash","","",1,[[]]]],"p":[[4,"PsdChannelCompression"],[4,"PsdChannelKind"],[4,"ColorMode"],[4,"PsdDepth"],[3,"PsdLayer"],[3,"Psd"]]};
addSearchOptions(searchIndex);initSearch(searchIndex);