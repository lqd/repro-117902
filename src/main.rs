fn main() {
    let context = wgpu::core::global::Global::new(
        wgpu::core::identity::IdentityManagerFactory,
        Default::default(),
    );
    let adapter_id = context
        .request_adapter(wgpu::core::instance::AdapterInputs::Mask(
            wgpu::Backends::all(),
            |_| (),
        ));

    let device_id = context.adapter_request_device::<wgpu::core::api::Metal>(
        adapter_id,
        &Default::default(),
        (),
    );

    let texture_descriptor = wgpu::TextureDescriptor {
        size: Default::default(),
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        dimension: wgpu::TextureDimension::D2,
        label: None,
        mip_level_count: 1,
        sample_count: 1,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    };

    wgpu::device_create_texture(&context, &device_id, &texture_descriptor);
    panic!("non-deterministic failure should have happened above");
}
