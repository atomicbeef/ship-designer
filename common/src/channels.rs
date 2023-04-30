use num_enum::IntoPrimitive;

#[derive(IntoPrimitive)]
#[repr(usize)]
pub enum Channel {
    PlayerConnectionEvents,
    PartCommands,
    Missile,
}