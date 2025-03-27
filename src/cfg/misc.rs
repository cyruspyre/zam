use inkwell::targets::{InitializationConfig, Target, TargetMachine, TargetMachineOptions};

pub fn size() -> u32 {
    Target::initialize_native(&InitializationConfig::default()).unwrap();

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).unwrap();
    let machine = target
        .create_target_machine_from_options(&triple, TargetMachineOptions::new())
        .unwrap();

    machine.get_target_data().get_pointer_byte_size(None) * 8
}
