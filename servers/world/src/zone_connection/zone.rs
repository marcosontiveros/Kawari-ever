//! All things zone related, such as changing the weather or warping.

use crate::{
    InstanceQuery, ObsfucationData, ToServer, ZoneConnection, inventory::BuyBackList, lua::LuaZone,
    zone_connection::TeleportQuery,
};
use kawari::{
    common::{
        EstateType, FestivalId, HandlerId, HousingFlag, PlotState, Position, TerritoryIntendedUse,
        WarpType, internal_housing_row, timestamp_secs,
    },
    constants::OBFUSCATION_ENABLED_MODE,
    ipc::zone::{
        ActorControlCategory, ApartmentBuildingState, Condition, DutyFinderSetting, Furniture,
        FurnitureList, House, HouseList, ServerZoneIpcData, ServerZoneIpcSegment,
        UpdateMapMarkersForWardDivision, WardDivision, WardDivisionInfo, WardMapInfo,
        WardMapInfoFlags, ZoneInit, ZoneInitFlags,
    },
    packet::{ConnectionState, PacketSegment, ScramblerKeyGenerator, SegmentData, SegmentType},
};

impl ZoneConnection {
    /// Request to change the zone to a given TerritoryType and position/rotation.
    pub async fn change_zone(
        &mut self,
        new_zone_id: u16,
        new_position: Option<Position>,
        new_rotation: Option<f32>,
        warp_type: WarpType,
        loading_screen_vfx_id: u16,
    ) {
        self.handle
            .send(ToServer::ChangeZone(
                self.id,
                self.player_data.character.actor_id,
                new_zone_id,
                new_position,
                new_rotation,
                warp_type,
                loading_screen_vfx_id,
            ))
            .await;
    }

    /// Completes the zone change.
    pub async fn handle_zone_change(
        &mut self,
        exit_position: Position,
        exit_rotation: f32,
        lua_zone: &LuaZone,
        handler_id: Option<HandlerId>,
    ) {
        // Reset data
        self.spawned_in = false;
        self.fate_motivation_npcs.clear();
        self.content_handler_id = handler_id;
        self.player_data.buyback_list = BuyBackList::default();
        self.synced_level = None;

        let bound_by_duty = matches!(lua_zone.query, InstanceQuery::InstancedContent { .. });

        // Commit back our zone id and other volatile info on zone change.
        {
            self.player_data.volatile.zone_id = lua_zone.zone_id as i32;
            self.player_data.volatile.position = exit_position;
            self.player_data.volatile.rotation = exit_rotation as f64;

            let mut db = self.database.lock();
            db.commit_volatile(&self.player_data);
        }

        // Generate obsfucation-related keys if needed.
        if self.config.enable_packet_obsfucation {
            let seed1 = fastrand::u8(..);
            let seed2 = fastrand::u8(..);
            let seed3 = fastrand::u32(..);

            let generator = ScramblerKeyGenerator::new();

            self.obsfucation_data = ObsfucationData {
                seed1,
                seed2,
                seed3,
            };

            let ConnectionState::Zone { scrambler_keys, .. } = &mut self.state else {
                panic!("Unexpected connection type!");
            };
            *scrambler_keys = Some(generator.generate(seed1, seed2, seed3));

            tracing::info!(
                "You enabled packet obsfucation in your World config, if things break please report it!",
            );
        }

        // they send the initialize packet again for some reason
        self.send_segment(PacketSegment {
            segment_type: SegmentType::Initialize,
            data: SegmentData::Initialize {
                actor_id: self.player_data.character.actor_id,
                timestamp: timestamp_secs(),
            },
            ..Default::default()
        })
        .await;

        // Send obsfucation init
        let ipc = if self.config.enable_packet_obsfucation {
            ServerZoneIpcSegment::new(ServerZoneIpcData::InitializeObfuscation {
                unk_before: [0; 6],
                obsfucation_mode: OBFUSCATION_ENABLED_MODE,
                seed1: !self.obsfucation_data.seed1,
                seed2: !self.obsfucation_data.seed2,
                seed3: !self.obsfucation_data.seed3,
            })
        } else {
            ServerZoneIpcSegment::new(ServerZoneIpcData::InitializeObfuscation {
                unk_before: [0; 6],
                obsfucation_mode: 0,
                seed1: 0,
                seed2: 0,
                seed3: 0,
            })
        };
        self.send_ipc_self(ipc).await;

        self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::InitResponse {
            actor_id: self.player_data.character.actor_id,
        }))
        .await;

        // Send owned housing list (unsure where this fits in before ZoneInit?!)
        {
            let owned_housing;
            {
                let mut db = self.database.lock();
                owned_housing = db.find_owned_housing(self.player_data.character.content_id as u64);
            }

            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::OwnedHousing(owned_housing));
            self.send_ipc_self(ipc).await;
        }

        // Send the list of available items if we're in an inn, since its only accessible via their beds.
        if lua_zone.intended_use == TerritoryIntendedUse::Inn {
            let display_ids;
            {
                let mut game_data = self.game_data.lock();
                display_ids = game_data.get_latest_fittingshop_display_ids();
            }

            let ipc =
                ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateFittingShop { display_ids });
            self.send_ipc_self(ipc).await;
        }

        let mut game_festival_ids = self.config.active_festivals.map(FestivalId);
        let mut game_festival_phases = [0; 8];

        // Set up ocean fishing
        if let InstanceQuery::InstancedContent {
            content_finder_condition_id,
            ..
        } = lua_zone.query
        {
            let mut game_data = self.game_data.lock();
            if let Some(spots) =
                game_data.lookup_ikd_route_spots_via_content(content_finder_condition_id as u32)
            {
                // Special festivals used for ocean fishing.
                game_festival_ids = [
                    FestivalId(101),
                    FestivalId(102),
                    FestivalId::default(),
                    FestivalId::default(),
                    FestivalId::default(),
                    FestivalId::default(),
                    FestivalId::default(),
                    FestivalId::default(),
                ];

                game_festival_phases = [
                    spots[0] as u16 + 1, // IKDSpot + 1
                    23,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ];
            }
        }

        // Init Zone
        {
            let mut flags = if self.initial_login {
                ZoneInitFlags::INITIAL_LOGIN
            } else if bound_by_duty {
                ZoneInitFlags::INSTANCED_CONTENT | ZoneInitFlags::UNK5
            } else {
                ZoneInitFlags::default()
            };

            if let InstanceQuery::InstancedContent {
                content_finder_condition_id,
                ..
            } = lua_zone.query
            {
                self.offered_teleport = None; // Discard any previously offered teleports once we're in a duty.

                let mut game_data = self.game_data.lock();
                if game_data.is_duty_recorder_allowed(content_finder_condition_id) {
                    flags |= ZoneInitFlags::ENABLE_RECORD_READY_CHECK;
                }
            } else {
                flags |= ZoneInitFlags::ENABLE_FLYING;
            }

            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ZoneInit(ZoneInit {
                territory_type: lua_zone.zone_id,
                weather_id: lua_zone.weather_id,
                flags,
                content_finder_condition_id: if let InstanceQuery::InstancedContent {
                    content_finder_condition_id,
                    ..
                } = lua_zone.query
                {
                    content_finder_condition_id
                } else {
                    0
                },
                game_festival_ids,
                game_festival_phases,
                ui_festival_ids: self.config.active_festivals.map(FestivalId),
                ..Default::default()
            }));
            self.send_ipc_self(ipc).await;
        }

        let level;
        {
            let mut game_data = self.game_data.lock();

            level = self
                .player_data
                .inventory
                .equipped
                .calculate_item_level(&mut game_data) as u32;
        }

        self.actor_control_self(ActorControlCategory::SetItemLevel { level })
            .await;

        if !bound_by_duty {
            self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::DailyQuests {
                unk1: [0; 56],
            }))
            .await;

            self.send_ipc_self(ServerZoneIpcSegment::new(
                ServerZoneIpcData::DailyQuestRepeatFlags { unk1: [0; 8] },
            ))
            .await;
        }

        if self.initial_login {
            self.send_quest_information().await;
            self.send_crafting_gathering_information().await;
        }

        if lua_zone.intended_use == TerritoryIntendedUse::HousingOutdoor
            && let InstanceQuery::Ward { ward } = lua_zone.query
        {
            self.populate_ward(lua_zone.zone_id, ward).await;
        }

        if lua_zone.intended_use == TerritoryIntendedUse::HousingIndoor
            && let InstanceQuery::House { house_id } = lua_zone.query
        {
            // TODO: Reject doing anything if we entered an apartment lobby and somehow enter this code; those never have furniture and should be TerritoryIntendedUse::HousingIndoor too
            let interior_details;
            let mut interior_furniture;
            {
                let mut db = self.database.lock();
                interior_details = db.find_house_interior_details(house_id);
                interior_furniture = db.find_house_furniture(house_id, true, false);
            }

            self.send_ipc_self(ServerZoneIpcSegment::new(
                ServerZoneIpcData::HousingInteriorDetails(interior_details.unwrap_or_default()),
            ))
            .await;

            // TODO: Limit this by interior size soon...
            for index in 0..6 {
                let mut furniture: Vec<Furniture> = Vec::new();
                // Take up to 100 items.
                furniture.extend(
                    interior_furniture
                        .drain(0..std::cmp::min(interior_furniture.len(), Furniture::COUNT)),
                );

                self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::FurnitureList(
                    FurnitureList {
                        id: house_id,
                        count: 6,
                        index,
                        estate_type: EstateType::ApartmentRoom,
                        unk2: 100, // Indoors
                        furniture,
                    },
                )))
                .await;
            }
        }

        self.conditions
            .toggle_condition(Condition::BoundByDuty, bound_by_duty);
        self.conditions
            .toggle_condition(Condition::BoundByDuty56, bound_by_duty);

        self.send_conditions().await;
        self.update_class_info().await;
        self.send_stats().await;

        // Inform the server we're ready for it to send director data
        self.handle
            .send(ToServer::ReadyDirectorData(
                self.player_data.character.actor_id,
            ))
            .await;

        self.initial_login = false;
    }

    pub async fn populate_ward(&mut self, zone_id: u16, ward_id: u16) {
        // First, send some house info!
        let mut houses = [House::default(); 30];
        let mut division_map_markers = [WardMapInfo::default(); 30];
        let division_info = WardDivisionInfo {
            apartment_building_state: ApartmentBuildingState::IsOwn, // TODO: Get our actual apartment state from the db
            ward_division: if self.entering_subdivision {
                WardDivision::SubDivision
            } else {
                WardDivision::MainDivision
            },
        };

        let furniture;
        {
            let mut database = self.database.lock();
            let mut gamedata = self.game_data.lock();
            let all_houses = database.find_houses_in_ward(&mut gamedata, zone_id, ward_id);
            // TODO: Furniture is always empty for now, see database/housing.rs
            furniture =
                database.find_furniture_in_ward(zone_id, ward_id, self.entering_subdivision);

            // First get the ranges of the division we're entering, and the opposite division we're NOT entering.
            // `database.find_houses_in_ward` returns all 60 plots in order, so we can safely divide it up in halves.
            let (main_range, opposite_range) = if self.entering_subdivision {
                (30..all_houses.len(), 0..30)
            } else {
                (0..30, 30..all_houses.len())
            };

            // Copy house info for the division we're entering.
            houses[..].copy_from_slice(&all_houses[main_range]);

            // Finally, grab house icon info for the opposite division.
            for (house, marker) in all_houses[opposite_range]
                .iter()
                .zip(division_map_markers.iter_mut())
            {
                let mut flags = if house.status != PlotState::UnownedLand {
                    WardMapInfoFlags::HAS_OWNER
                } else {
                    WardMapInfoFlags::default()
                };

                if (house.flags & HousingFlag::OWNED_BY_FC) == HousingFlag::OWNED_BY_FC {
                    flags |= WardMapInfoFlags::IS_FREE_COMPANY;
                }

                *marker = WardMapInfo {
                    state: house.status,
                    size: house.plot_size,
                    flags,
                };
            }
        }

        let Some(land_id) = internal_housing_row(zone_id) else {
            tracing::error!(
                "populate_ward: attempted to find an internal housing row for non-housing zone {zone_id}!?"
            );
            // TODO: Send blank info? What should we do in this error state?
            return;
        };

        self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::HouseList(
            HouseList {
                land_id: land_id as u16,
                ward: ward_id,
                territory_type_id: zone_id,
                world_id: self.config.world_id,
                division_info,
                houses,
            },
        )))
        .await;

        self.send_ipc_self(ServerZoneIpcSegment::new(
            ServerZoneIpcData::UpdateMapMarkersForWardDivision(UpdateMapMarkersForWardDivision {
                ward_division_info: WardDivisionInfo {
                    apartment_building_state: ApartmentBuildingState::None, // TODO: Get the apartment state for the opposite division
                    ward_division: if self.entering_subdivision {
                        WardDivision::MainDivision
                    } else {
                        WardDivision::SubDivision
                    },
                },
                info: division_map_markers,
            }),
        ))
        .await;

        // Finally, populate the exterior furniture.
        // TODO: Actually send some real furniture, once we can do that!
        for index in 0..15 {
            self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::FurnitureList(
                FurnitureList {
                    estate_type: EstateType::Ward,
                    count: 15,
                    index,
                    furniture: furniture.clone(),
                    ..Default::default()
                },
            )))
            .await;
        }

        self.entering_subdivision = false; // TODO: Not sure if we'll need to reset this here but it should be okay for now
    }

    /// Request to teleport the player to the given `warp_id`, which is an index in the Warp Excel sheet.
    pub async fn warp(&mut self, warp_id: u32) {
        self.handle
            .send(ToServer::Warp(self.player_data.character.actor_id, warp_id))
            .await;
    }

    /// Request to teleport the player to the given `aetheryte_id`, which is an index in the Aetheryte or HousingAethernet Excel sheet.
    pub async fn warp_aetheryte(&mut self, aetheryte_id: u32, zone: &LuaZone) {
        let (destination_instance_id, destination_zone_id);
        {
            let mut game_data = self.game_data.lock();
            (destination_instance_id, destination_zone_id) = game_data
                .get_aetheryte(aetheryte_id)
                .expect("Failed to find the aetheryte!");
        }

        self.handle
            .send(ToServer::WarpPopRange(
                self.player_data.character.actor_id,
                destination_zone_id,
                destination_instance_id,
            ))
            .await;

        if self.can_offer_teleport(aetheryte_id) {
            let teleport_info = TeleportQuery {
                aetheryte_id: aetheryte_id as u16,
                // TODO: Verify if this does get sent at all to other players
                unk_flags: 0,
            };

            self.handle
                .send(ToServer::OfferTeleportToParty(
                    Some(self.party_id),
                    self.player_data.character.actor_id,
                    self.player_data.volatile.zone_id as u16,
                    teleport_info,
                ))
                .await;
        }

        self.entering_subdivision = false;

        // If we're using the housing aethernet, we need to inform the client about the division they're going to.
        if let InstanceQuery::Ward { ward } = zone.query {
            {
                let mut gamedata = self.game_data.lock();
                if gamedata.is_subdivision_aetheryte(aetheryte_id) {
                    self.entering_subdivision = true;
                }
            }
            self.populate_ward(self.player_data.volatile.zone_id as u16, ward)
                .await;
        }
    }

    pub async fn discover_location(&mut self, map_id: u32, map_part_id: u32) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::LocationDiscovered {
            map_id,
            map_part_id,
        });
        self.send_ipc_self(ipc).await;
    }

    pub async fn join_content(&mut self, id: u16, settings: DutyFinderSetting) {
        // Store our old information, for when we leave the instance
        self.old_zone_id = self.player_data.volatile.zone_id as u16;
        self.old_position = self.player_data.volatile.position;
        self.old_rotation = self.player_data.volatile.rotation as f32;

        self.handle
            .send(ToServer::JoinContent(
                self.player_data.character.actor_id,
                id,
                settings,
            ))
            .await;
    }

    /// Changes the time of day in the current zone for our client.
    pub async fn set_eorzean_time(&mut self, offset: i64) {
        self.send_ipc_self(ServerZoneIpcSegment::new(
            ServerZoneIpcData::EorzeanTimeOffset { offset },
        ))
        .await;
    }
}
