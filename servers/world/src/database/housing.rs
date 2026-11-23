use diesel::prelude::*;

use super::{WorldDatabase, models, schema};
use crate::GameData;
use crate::database::House;
use kawari::common::{HouseId, HousingFlag, LandData, LandFlag, PlotState, internal_housing_row};
use kawari::ipc::zone::{OwnedHousing, PlotSize};

impl WorldDatabase {
    pub fn find_house(&mut self, the_house_id: HouseId) -> Option<House> {
        use schema::house::dsl::*;
        let the_id = i64::from(the_house_id);

        house
            .filter(house_id.eq(the_id))
            .first::<House>(&mut self.connection)
            .ok()
    }

    /// Returns all houses in the entire ward (including subdivision).
    pub fn find_houses_in_ward(
        &mut self,
        game_data: &mut GameData,
        zone_id: u16,
        ward_number: u16,
    ) -> [kawari::ipc::zone::House; 60] {
        use schema::house::dsl::*;

        let mut estates = [kawari::ipc::zone::House::default(); 60];

        if let Ok(the_houses) = house
            .select(models::House::as_select())
            .load(&mut self.connection)
        {
            for the_house in the_houses {
                let the_house_id: HouseId = the_house.house_id.into();

                if (the_house_id.territory_type_id == zone_id)
                    && (the_house_id.ward_index as u16 == ward_number)
                    && !the_house_id.unit.apartment_flag
                {
                    let exterior = self
                        .find_house_exterior_details(the_house_id)
                        .unwrap_or_default();
                    let plot_number = the_house_id.unit.apartment_division_plot_index as usize;

                    let internal_row = internal_housing_row(zone_id);

                    let mut plot_size = PlotSize::default();

                    if let Some(internal_row) = internal_row
                        && let Some(land_sets) = game_data.get_land_sets(internal_row)
                    {
                        plot_size = PlotSize::from_repr(land_sets[plot_number].PlotSize)
                            .unwrap_or_default();
                    }

                    let mut flags = HousingFlag::LOCKED;
                    if the_house.is_free_company {
                        flags |= HousingFlag::OWNED_BY_FC;
                    }
                    if !the_house.is_locked {
                        flags |= HousingFlag::OPEN;
                    }

                    estates[plot_number] = kawari::ipc::zone::House {
                        status: if exterior.is_under_construction() {
                            PlotState::OwnedLand
                        } else {
                            PlotState::OwnedEstate
                        },
                        plot_size,
                        flags,
                        exterior,
                        ..Default::default() // TODO: free companies
                    };
                }
            }
        }

        estates
    }

    /// Returns the interior details for one house.
    pub fn find_house_interior_details(
        &mut self,
        the_house_id: HouseId,
    ) -> Option<kawari::ipc::zone::HousingInteriorDetails> {
        use schema::house_interior::dsl::*;
        let the_id = i64::from(the_house_id);

        if let Ok(the_interior) = house_interior
            .filter(house_id.eq(the_id))
            .first::<models::HouseInterior>(&mut self.connection)
        {
            let interior: kawari::ipc::zone::HousingInteriorDetails =
                serde_json::from_str(&the_interior.details).unwrap_or_default();
            return Some(interior);
        }

        None
    }

    /// Returns the exterior details for one house.
    pub fn find_house_exterior_details(
        &mut self,
        the_house_id: HouseId,
    ) -> Option<kawari::ipc::zone::HouseExterior> {
        use schema::house_exterior::dsl::*;
        let the_id = i64::from(the_house_id);
        if let Ok(the_exterior) = house_exterior
            .filter(house_id.eq(the_id))
            .first::<models::HouseExterior>(&mut self.connection)
        {
            let exterior: kawari::ipc::zone::HouseExterior =
                serde_json::from_str(&the_exterior.details).unwrap_or_default();
            return Some(exterior);
        }

        None
    }

    /// Returns the furniture for a house's exterior or interior from its inventory or storeroom as full Items (...eventually).
    // TODO: kawari::ipc::zone::Furniture is not the correct type for this, it needs to be stored as full Items with extra position/rotation & model data. The inventory and storerooms in-game store actual Items, and the player can retrieve them back.
    pub fn find_house_furniture(
        &mut self,
        the_house_id: HouseId,
        want_interior: bool,
        want_storeroom: bool,
    ) -> Vec<kawari::ipc::zone::Furniture> {
        use schema::house_exterior::dsl::*;
        use schema::house_interior::dsl::*;

        let the_id = i64::from(the_house_id);
        let mut the_furniture: Vec<kawari::ipc::zone::Furniture> = Vec::new();

        if want_interior {
            if let Ok(the_interior) = house_interior
                .filter(schema::house_interior::dsl::house_id.eq(the_id))
                .first::<models::HouseInterior>(&mut self.connection)
            {
                let selected_inventory = if !want_storeroom {
                    the_interior.inventory
                } else {
                    the_interior.storeroom
                };
                the_furniture = serde_json::from_str(&selected_inventory).unwrap_or_default();
            }
        } else {
            if let Ok(the_exterior) = house_exterior
                .filter(schema::house_exterior::dsl::house_id.eq(the_id))
                .first::<models::HouseExterior>(&mut self.connection)
            {
                let selected_inventory = if !want_storeroom {
                    the_exterior.inventory
                } else {
                    the_exterior.storeroom
                };
                the_furniture = serde_json::from_str(&selected_inventory).unwrap_or_default();
            }
        }

        the_furniture
    }

    // Returns the furniture for every house in the ward.
    // TODO: This is dummy code temporarily. Once the Item type is decided on, this function will call `find_house_furniture` in a loop and convert the Item type to kawari::ipc::zone::Furniture.
    pub fn find_furniture_in_ward(
        &mut self,
        _zone_id: u16,
        _ward_number: u16,
        _want_subdivision: bool,
    ) -> Vec<kawari::ipc::zone::Furniture> {
        Vec::new()
    }

    // Adds a new estate to the database.
    // TODO: Free companies' estates, later on down the road.
    pub fn add_house(&mut self, the_house_id: HouseId, for_owner_id: u64) {
        use schema::house::dsl::*;
        use schema::house_exterior::dsl::*;
        use schema::house_interior::dsl::*;

        let the_id = i64::from(the_house_id);
        let the_owner = for_owner_id as i64;
        let the_house = models::House {
            house_id: the_id,
            owner_id: the_owner,
            ..Default::default()
        };

        diesel::insert_into(house)
            .values(the_house)
            .execute(&mut self.connection)
            .unwrap();

        let the_exterior = models::HouseExterior {
            house_id: the_id,
            ..Default::default()
        };

        diesel::insert_into(house_exterior)
            .values(the_exterior)
            .execute(&mut self.connection)
            .unwrap();

        let the_interior = models::HouseInterior {
            house_id: the_id,
            ..Default::default()
        };

        diesel::insert_into(house_interior)
            .values(the_interior)
            .execute(&mut self.connection)
            .unwrap();
    }

    // Removes a house from the database.
    // TODO: Eventually, also shared tenants and the ban list
    pub fn remove_house(&mut self, the_house_id: HouseId, for_owner_id: u64) {
        use schema::house::dsl::*;
        use schema::house_exterior::dsl::*;
        use schema::house_interior::dsl::*;

        let the_id = i64::from(the_house_id);

        diesel::delete(house_exterior.filter(schema::house_exterior::house_id.eq(the_id)))
            .execute(&mut self.connection)
            .unwrap();

        diesel::delete(house_interior.filter(schema::house_interior::house_id.eq(the_id)))
            .execute(&mut self.connection)
            .unwrap();

        diesel::delete(
            house
                .filter(schema::house::house_id.eq(the_id))
                .filter(owner_id.eq(for_owner_id as i64)),
        )
        .execute(&mut self.connection)
        .unwrap();
    }

    // Modifies a house interior's information, such as its inventories (TODO!), orchestrion song, or details.
    pub fn modify_interior(
        &mut self,
        the_house_id: HouseId,
        the_details: Option<kawari::ipc::zone::HousingInteriorDetails>,
        song_id: Option<u32>,
    ) {
        use schema::house_interior::dsl::*;

        let the_id = i64::from(the_house_id);

        if let Some(the_details) = the_details {
            diesel::update(house_interior)
                .filter(house_id.eq(the_id))
                .set(details.eq(serde_json::to_string(&the_details).unwrap()))
                .execute(&mut self.connection)
                .unwrap();
        }

        if let Some(song_id) = song_id {
            diesel::update(house_interior)
                .filter(house_id.eq(the_id))
                .set(orchestrion_song.eq(song_id as i32))
                .execute(&mut self.connection)
                .unwrap();
        }
    }

    // Modifies a house exterior's information, such as its inventories (TODO!) or details.
    pub fn modify_exterior(
        &mut self,
        the_house_id: HouseId,
        the_details: Option<kawari::ipc::zone::HouseExterior>,
    ) {
        use schema::house_exterior::dsl::*;

        let the_id = i64::from(the_house_id);

        if let Some(the_details) = the_details {
            diesel::update(house_exterior)
                .filter(house_id.eq(the_id))
                .set(details.eq(serde_json::to_string(&the_details).unwrap()))
                .execute(&mut self.connection)
                .unwrap();
        }
    }

    // Returns an OwnedHousing containing properties the given content id owns or has estates shared with.
    pub fn find_owned_housing(&mut self, for_content_id: u64) -> OwnedHousing {
        use schema::house::dsl::*;

        let mut owned_housing = OwnedHousing::default();

        if let Ok(the_houses) = house
            .filter(owner_id.eq(for_content_id as i64))
            .select(house_id)
            .load::<i64>(&mut self.connection)
        {
            for estate in the_houses {
                let the_house_id = HouseId::from(estate);

                // TODO: Don't hardcode the LandFlags
                if the_house_id.unit.apartment_flag {
                    owned_housing.apartment = LandData {
                        id: the_house_id,
                        land_flags: LandFlag::ESTATE_HALL_BUILT | LandFlag::TELEPORTABLE,
                        ..Default::default()
                    };
                } else {
                    owned_housing.personal = LandData {
                        id: the_house_id,
                        land_flags: LandFlag::ESTATE_HALL_BUILT | LandFlag::TELEPORTABLE,
                        ..Default::default()
                    };
                    // TODO: Free company estate, shared estates
                }
            }
        }

        owned_housing
    }
}
