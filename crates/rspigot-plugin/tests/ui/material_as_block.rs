use rspigot_plugin::{BlockPos, Material, World};

fn place(world: &World) {
    world.block(BlockPos { x: 0, y: 64, z: 0 }).set(Material::Diamond);
}

fn main() {}
