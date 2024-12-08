use crate::game::components::{Direction, Position, Speed, Velocity};

use super::{
    area::Area,
    components::{Color, Player, Size},
};

pub fn system_update_position(area: &mut Area) {
    for (_, (pos, vel)) in area.world.query_mut::<(&mut Position, &Velocity)>() {
        pos.0 += vel.0 * area.delta_time;
    }
}

pub fn system_update_velocity(area: &mut Area) {
    for (_, (vel, dir, speed)) in area
        .world
        .query_mut::<(&mut Velocity, &Direction, &Speed)>()
    {
        vel.0 = dir.0 * speed.0;
    }
}

pub fn system_render(area: &mut Area) {
    for (_, (pos, size, color, player)) in area
        .world
        .query_mut::<(&Position, &Size, &Color, &Player)>()
    {
        let x_bytes = pos.0.x.to_le_bytes();
        let y_bytes = pos.0.y.to_le_bytes();
        let size_bytes = size.0.to_le_bytes();
        let color_bytes = color.to_le_bytes();

        let mut data = Vec::new();

        data.extend_from_slice(&x_bytes);
        data.extend_from_slice(&y_bytes);
        data.extend_from_slice(&size_bytes);
        data.extend_from_slice(&color_bytes);

        let _ = player.connection.send_datagram(data);
    }
}
