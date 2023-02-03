use std::f32::consts::TAU;

use three_d::{prelude::*, Positions};

pub struct ConeBuilder<SectorCount, Radius, Height>
where
    SectorCount: Copy,
    Radius: Copy,
    Height: Copy,
{
    pub sector_count: SectorCount,
    pub radius: Radius,
    pub height: Height,
}

impl ConeBuilder<(), (), ()> {
    pub fn new() -> Self {
        Self {
            sector_count: (),
            radius: (),
            height: (),
        }
    }
}

impl<Radius, Height> ConeBuilder<(), Radius, Height>
where
    Radius: Copy,
    Height: Copy,
{
    pub fn with_sector_count(&self, sector_count: u32) -> ConeBuilder<u32, Radius, Height> {
        let Self { radius, height, .. } = *self;
        ConeBuilder {
            sector_count,
            radius,
            height,
        }
    }
}

impl<SectorCount, Height> ConeBuilder<SectorCount, (), Height>
where
    SectorCount: Copy,
    Height: Copy,
{
    pub fn with_radius(&self, radius: f32) -> ConeBuilder<SectorCount, f32, Height> {
        let Self {
            sector_count,
            height,
            ..
        } = *self;
        ConeBuilder {
            sector_count,
            radius,
            height,
        }
    }
}

impl<SectorCount, Radius> ConeBuilder<SectorCount, Radius, ()>
where
    SectorCount: Copy,
    Radius: Copy,
{
    pub fn with_height(&self, height: f32) -> ConeBuilder<SectorCount, Radius, f32> {
        let Self {
            sector_count,
            radius,
            ..
        } = *self;
        ConeBuilder {
            sector_count,
            radius,
            height,
        }
    }
}

impl ConeBuilder<u32, f32, f32> {
    pub fn build(self) -> Positions {
        let Self {
            sector_count,
            radius,
            height,
        } = self;
        let sector_turn_size = TAU / sector_count as f32;

        Positions::F32(
            (0..sector_count)
                .flat_map(|sector| {
                    let sector = sector as f32;

                    let sector_start = sector_turn_size * sector;
                    let start_vertex = vec3(
                        sector_start.sin() * radius,
                        -(height / 2.0),
                        sector_start.cos() * radius,
                    );

                    let sector_end = sector_turn_size * (sector + 1.0);
                    let end_vertex = vec3(
                        sector_end.sin() * radius,
                        -(height / 2.0),
                        sector_end.cos() * radius,
                    );

                    [vec3(0.0, height / 2.0, 0.0), start_vertex, end_vertex]
                })
                .collect(),
        )
    }
}
