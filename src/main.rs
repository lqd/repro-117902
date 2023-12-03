fn main() {
    let context = wgpu::core::global::Global::new(
        wgpu::core::identity::IdentityManagerFactory,
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
        ballast: [0u64; 4096],
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
    };

    wgpu::device_create_texture(&context, &device_id, &texture_descriptor);
    panic!("non-deterministic failure should have happened above");
}
