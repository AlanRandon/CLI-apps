use std::f32::consts::TAU;

use three_d::{prelude::*, Positions};

pub struct ConeBuilder<SectorCount, Radius, Height, Origin, Direction>
where
    SectorCount: Copy,
    Radius: Copy,
    Height: Copy,
    Origin: Copy,
    Direction: Copy,
{
    sector_count: SectorCount,
    radius: Radius,
    height: Height,
    origin: Origin,
    direction: Direction,
}

impl ConeBuilder<(), (), (), (), ()> {
    pub const fn new() -> Self {
        Self {
            sector_count: (),
            radius: (),
            height: (),
            origin: (),
            direction: (),
        }
    }
}

impl<Radius, Height, Origin, Direction> ConeBuilder<(), Radius, Height, Origin, Direction>
where
    Radius: Copy,
    Height: Copy,
    Origin: Copy,
    Direction: Copy,
{
    pub const fn with_sector_count(
        &self,
        sector_count: u32,
    ) -> ConeBuilder<u32, Radius, Height, Origin, Direction> {
        let Self {
            radius,
            height,
            origin,
            direction,
            ..
        } = *self;
        ConeBuilder {
            sector_count,
            radius,
            height,
            origin,
            direction,
        }
    }
}

impl<SectorCount, Height, Origin, Direction> ConeBuilder<SectorCount, (), Height, Origin, Direction>
where
    SectorCount: Copy,
    Height: Copy,
    Origin: Copy,
    Direction: Copy,
{
    pub const fn with_radius(
        &self,
        radius: f32,
    ) -> ConeBuilder<SectorCount, f32, Height, Origin, Direction> {
        let Self {
            sector_count,
            height,
            origin,
            direction,
            ..
        } = *self;
        ConeBuilder {
            sector_count,
            radius,
            height,
            origin,
            direction,
        }
    }
}

impl<SectorCount, Radius, Origin, Direction> ConeBuilder<SectorCount, Radius, (), Origin, Direction>
where
    SectorCount: Copy,
    Radius: Copy,
    Origin: Copy,
    Direction: Copy,
{
    pub const fn with_height(
        &self,
        height: f32,
    ) -> ConeBuilder<SectorCount, Radius, f32, Origin, Direction> {
        let Self {
            sector_count,
            radius,
            origin,
            direction,
            ..
        } = *self;
        ConeBuilder {
            sector_count,
            radius,
            height,
            origin,
            direction,
        }
    }
}

impl<SectorCount, Radius, Height, Direction> ConeBuilder<SectorCount, Radius, Height, (), Direction>
where
    SectorCount: Copy,
    Radius: Copy,
    Height: Copy,
    Direction: Copy,
{
    pub const fn with_origin(
        &self,
        origin: Vec3,
    ) -> ConeBuilder<SectorCount, Radius, Height, Vec3, Direction> {
        let Self {
            sector_count,
            radius,
            height,
            direction,
            ..
        } = *self;
        ConeBuilder {
            sector_count,
            radius,
            height,
            origin,
            direction,
        }
    }
}

impl<SectorCount, Radius, Height, Origin> ConeBuilder<SectorCount, Radius, Height, Origin, ()>
where
    SectorCount: Copy,
    Radius: Copy,
    Height: Copy,
    Origin: Copy,
{
    pub const fn with_direction(
        &self,
        direction: Vec3,
    ) -> ConeBuilder<SectorCount, Radius, Height, Origin, Vec3> {
        let Self {
            sector_count,
            radius,
            height,
            origin,
            ..
        } = *self;
        ConeBuilder {
            sector_count,
            radius,
            height,
            origin,
            direction,
        }
    }
}

impl ConeBuilder<u32, f32, f32, Vec3, Vec3> {
    pub fn build(self) -> Positions {
        let Self {
            sector_count,
            radius,
            height,
            origin,
            direction,
        } = self;
        let sector_turn_size = TAU / sector_count as f32;
        let direction = direction.normalize();
        let apex = origin + height * direction;
        let untransformed_apex = vec3(0.0, height, 0.0);
        let transform = rotation_matrix_from_dir_to_dir(untransformed_apex, apex);

        Positions::F32(
            (0..sector_count)
                .flat_map(|sector| {
                    let sector = sector as f32;

                    let sector_start = sector_turn_size * sector;
                    let start_vertex = vec3(
                        sector_start.sin() * radius,
                        0.0,
                        sector_start.cos() * radius,
                    );

                    let sector_end = sector_turn_size * (sector + 1.0);
                    let end_vertex =
                        vec3(sector_end.sin() * radius, 0.0, sector_end.cos() * radius);

                    let start_vertex = transform.transform_vector(start_vertex);
                    let end_vertex = transform.transform_vector(end_vertex);

                    [
                        apex,
                        start_vertex + origin,
                        end_vertex + origin,
                        origin,
                        start_vertex + origin,
                        end_vertex + origin,
                    ]
                })
                .collect(),
        )
    }
}
