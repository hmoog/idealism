use blockdag::BlockMetadataRef;

pub trait Config: virtual_voting::Config<Source = BlockMetadataRef<Self>> {}
