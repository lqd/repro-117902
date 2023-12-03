#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Empty = 0,
    Metal = 2,
    Gl = 5,
}
bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Copy, Clone, PartialEq, Eq, )]
    pub struct Backends : u32 {
        const METAL = 1 << Backend::Metal as u32;
    }
}
impl From<Backend> for Backends {
    fn from(backend: Backend) -> Self {
        Self::from_bits(1 << backend as u32).unwrap()
    }
}
bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Default, Copy , Clone , PartialEq , Eq , )]
    pub struct Features : u64 {
        const DEPTH_CLIP_CONTROL = 1 << 0;
        const TIMESTAMP_QUERY = 1 << 1;
        const INDIRECT_FIRST_INSTANCE = 1 << 2;
        const SHADER_F16 = 1 << 8;
        const RG11B10UFLOAT_RENDERABLE = 1 << 23;
        const DEPTH32FLOAT_STENCIL8 = 1 << 24;
        const TEXTURE_COMPRESSION_BC = 1 << 25;
        const TEXTURE_COMPRESSION_ETC2 = 1 << 26;
        const TEXTURE_COMPRESSION_ASTC = 1 << 27;
        const TEXTURE_FORMAT_16BIT_NORM = 1 << 29;
        const TEXTURE_COMPRESSION_ASTC_HDR = 1 << 30;
        const TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES = 1 << 31;
        const PIPELINE_STATISTICS_QUERY = 1 << 32;
        const TIMESTAMP_QUERY_INSIDE_PASSES = 1 << 33;
        const MAPPABLE_PRIMARY_BUFFERS = 1 << 34;
        const TEXTURE_BINDING_ARRAY = 1 << 35;
        const BUFFER_BINDING_ARRAY = 1 << 36;
        const STORAGE_RESOURCE_BINDING_ARRAY = 1 << 37;
        const SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING = 1 << 38;
        const UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING = 1 << 39;
        const PARTIALLY_BOUND_BINDING_ARRAY = 1 << 40;
        const MULTI_DRAW_INDIRECT = 1 << 41;
        const MULTI_DRAW_INDIRECT_COUNT = 1 << 42;
        const PUSH_CONSTANTS = 1 << 43;
        const ADDRESS_MODE_CLAMP_TO_ZERO = 1 << 44;
        const ADDRESS_MODE_CLAMP_TO_BORDER = 1 << 45;
        const POLYGON_MODE_LINE = 1 << 46;
        const POLYGON_MODE_POINT = 1 << 47;
        const CONSERVATIVE_RASTERIZATION = 1 << 48;
        const VERTEX_WRITABLE_STORAGE = 1 << 49;
        const CLEAR_TEXTURE = 1 << 50;
        const SPIRV_SHADER_PASSTHROUGH = 1 << 51;
        const MULTIVIEW = 1 << 52;
        const VERTEX_ATTRIBUTE_64BIT = 1 << 53;
        const SHADER_F64 = 1 << 59;
        const SHADER_I16 = 1 << 60;
        const SHADER_PRIMITIVE_INDEX = 1 << 61;
        const SHADER_EARLY_DEPTH_TEST = 1 << 62;
    }
}
#[derive(Clone, Default)]
pub struct DeviceDescriptor {
    pub features: Features,
}
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AstcBlock {
    B12x12,
}
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AstcChannel {
    Unorm,
    Hdr,
}
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TextureFormat {
    Rgba8UnormSrgb, //
    Rgba8Snorm,
    Rgba8Uint,
    Rgba8Sint,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    Rgb9e5Ufloat,
    Rgb10a2Unorm,
    Rg11b10Float,
    Rg32Uint,
    Rg32Sint,
    Rg32Float,
    Rgba16Uint,
    Rgba16Sint,
    Rgba16Unorm,
    Rgba16Snorm,
    Rgba16Float,
    Rgba32Uint,
    Rgba32Sint,
    Rgba32Float,
    Stencil8,
    Depth16Unorm,
    Depth24Plus,
    Depth24PlusStencil8,
    Depth32Float,
    Depth32FloatStencil8,
    Bc1RgbaUnorm,
    Bc1RgbaUnormSrgb,
    Bc2RgbaUnorm,
    Bc2RgbaUnormSrgb,
    Bc3RgbaUnorm,
    Bc3RgbaUnormSrgb,
    Bc4RUnorm,
    Bc4RSnorm,
    Bc5RgUnorm,
    Bc5RgSnorm,
    Bc6hRgbUfloat,
    Bc6hRgbFloat,
    Bc7RgbaUnorm,
    Bc7RgbaUnormSrgb,
    Etc2Rgb8Unorm,
    Etc2Rgb8UnormSrgb,
    Etc2Rgb8A1Unorm,
    Etc2Rgb8A1UnormSrgb,
    Etc2Rgba8Unorm,
    Etc2Rgba8UnormSrgb,
    EacR11Unorm,
    EacR11Snorm,
    EacRg11Unorm,
    EacRg11Snorm,
    Astc {
        block: AstcBlock,
        channel: AstcChannel,
    },
}
impl TextureFormat {
    pub fn required_features(&self) -> Features {
        match *self {
            Self::Rgba8UnormSrgb
            | Self::Rgba8Snorm
            | Self::Rgba8Uint
            | Self::Rgba8Sint
            | Self::Bgra8Unorm
            | Self::Bgra8UnormSrgb
            | Self::Rgb9e5Ufloat
            | Self::Rgb10a2Unorm
            | Self::Rg11b10Float
            | Self::Rg32Uint
            | Self::Rg32Sint
            | Self::Rg32Float
            | Self::Rgba16Uint
            | Self::Rgba16Sint
            | Self::Rgba16Float
            | Self::Rgba32Uint
            | Self::Rgba32Sint
            | Self::Rgba32Float
            | Self::Stencil8
            | Self::Depth16Unorm
            | Self::Depth24Plus
            | Self::Depth24PlusStencil8
            | Self::Depth32Float => Features::empty(),
            Self::Depth32FloatStencil8 => Features::DEPTH32FLOAT_STENCIL8,
            Self::Rgba16Unorm | Self::Rgba16Snorm => Features::TEXTURE_FORMAT_16BIT_NORM,
            Self::Bc1RgbaUnorm
            | Self::Bc1RgbaUnormSrgb
            | Self::Bc2RgbaUnorm
            | Self::Bc2RgbaUnormSrgb
            | Self::Bc3RgbaUnorm
            | Self::Bc3RgbaUnormSrgb
            | Self::Bc4RUnorm
            | Self::Bc4RSnorm
            | Self::Bc5RgUnorm
            | Self::Bc5RgSnorm
            | Self::Bc6hRgbUfloat
            | Self::Bc6hRgbFloat
            | Self::Bc7RgbaUnorm
            | Self::Bc7RgbaUnormSrgb => Features::TEXTURE_COMPRESSION_BC,
            Self::Etc2Rgb8Unorm
            | Self::Etc2Rgb8UnormSrgb
            | Self::Etc2Rgb8A1Unorm
            | Self::Etc2Rgb8A1UnormSrgb
            | Self::Etc2Rgba8Unorm
            | Self::Etc2Rgba8UnormSrgb
            | Self::EacR11Unorm
            | Self::EacR11Snorm
            | Self::EacRg11Unorm
            | Self::EacRg11Snorm => Features::TEXTURE_COMPRESSION_ETC2,
            Self::Astc { channel, .. } => match channel {
                AstcChannel::Hdr => Features::TEXTURE_COMPRESSION_ASTC_HDR,
                AstcChannel::Unorm  => Features::TEXTURE_COMPRESSION_ASTC,
            },
        }
    }
    pub fn guaranteed_format_features(&self, _device_features: Features) {
        unimplemented!()
    }
}
pub struct TextureDescriptor {
    pub ballast: [u64; 4096], // Something big that comes first in layout.
                              // repr(C) or big alignment works.
    pub format: TextureFormat,
}
