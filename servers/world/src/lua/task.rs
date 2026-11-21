use crate::{
    RemakeMode, ToServer,
    inventory::{CrystalKind, CurrencyKind},
};
use kawari::{
    common::Position,
    ipc::zone::{EventType, GrandCompany, SceneFlags, ServerZoneIpcSegment},
    packet::PacketSegment,
};
use physis::savedata::chardat::CustomizeData;

#[derive(Clone, Debug)]
pub enum LuaTask {
    ChangeTerritory {
        zone_id: u16,
        exit_position: Option<Position>,
        exit_rotation: Option<f32>,
    },
    SetRemakeMode(RemakeMode),
    Warp {
        warp_id: u32,
    },
    BeginLogOut,
    FinishEvent {
        nested_args: Vec<i32>,
    },
    UnlockClassJob {
        classjob_id: u8,
    },
    WarpAetheryte {
        aetheryte_id: u32,
    },
    ToggleInvisibility {
        invisible: bool,
    },
    Unlock {
        id: u32,
    },
    UnlockAll,
    UnlockAetheryte {
        id: u32,
        on: bool,
    },
    SetLevel {
        level: u16,
    },
    ModifyCurrency {
        id: CurrencyKind,
        amount: i32,
        send_client_update: bool,
    },
    ModifyCrystal {
        id: CrystalKind,
        amount: i32,
        send_client_update: bool,
    },
    GmSetOrchestrion {
        value: bool,
        id: u32,
    },
    ToggleOrchestrion {
        id: u32,
    },
    AddItem {
        id: u32,
        quantity: i32,
        send_client_update: bool,
    },
    UnlockContent {
        id: u16,
    },
    UnlockAllContent,
    AddExp {
        amount: i32,
    },
    StartEvent {
        event_id: u32,
        event_type: EventType,
        event_arg: u32,
    },
    SetInnWakeup {
        watched: bool,
    },
    ToggleMount {
        id: u32,
    },
    ToggleGlassesStyle {
        id: u32,
    },
    ToggleGlassesStyleAll,
    ToggleOrnament {
        id: u32,
    },
    ToggleOrnamentAll,
    UnlockBuddyEquip {
        id: u32,
    },
    UnlockBuddyEquipAll,
    ToggleChocoboTaxiStand {
        id: u32,
    },
    ToggleChocoboTaxiStandAll,
    ToggleCaughtFish {
        id: u32,
    },
    ToggleCaughtFishAll,
    ToggleCaughtSpearfish {
        id: u32,
    },
    ToggleCaughtSpearfishAll,
    ToggleTripleTriadCard {
        id: u32,
    },
    ToggleTripleTriadCardAll,
    ToggleAdventure {
        id: u32,
    },
    ToggleAdventureAll,
    ToggleCutsceneSeen {
        id: u32,
        value: bool,
    },
    ToggleCutsceneSeenAll,
    ToggleMinion {
        id: u32,
    },
    ToggleMinionAll,
    ToggleAetherCurrent {
        id: u32,
    },
    ToggleAetherCurrentAll,
    ToggleAetherCurrentCompFlgSet {
        id: u32,
    },
    ToggleAetherCurrentCompFlgSetAll,
    // previously, this was kept as a separate thing apart from tasks
    // but I discovered that this doesn't mix well - for example with play_scene (segment-based) and start_event (task)
    // this is because segments were always sent before tasks and there wasn't strong ordering
    // so be careful when changing this system!
    SendSegment {
        segment: PacketSegment<ServerZoneIpcSegment>,
    },
    // NOTE: this is mostly a workaround in a limitation in the scripting API
    StartTalkEvent,
    AcceptQuest {
        id: u32,
    },
    FinishQuest {
        id: u32,
    },
    GainStatusEffect {
        effect_id: u16,
        effect_param: u16,
        duration: f32,
    },
    RegisterForContent {
        content_id: u16,
    },
    QuestSequence {
        id: u32,
        sequence: u8,
    },
    CancelQuest {
        id: u32,
    },
    IncompleteQuest {
        id: u32,
    },
    AbandonContent,
    SetHomepoint {
        homepoint: u16,
    },
    ReturnToHomepoint,
    JoinContent {
        id: u32,
    },
    FinishCastingGlamour,
    PlayScene {
        scene: u16,
        scene_flags: SceneFlags,
        params: Vec<u32>,
    },
    ResumeEvent {
        scene: u16,
        resume_id: u8,
        params: Vec<u32>,
    },
    ToggleHowTo {
        value: bool,
        id: u32,
    },
    SendMailboxStatus,
    SetGrandCompany {
        company: GrandCompany,
    },
    SetGrandCompanyRank {
        rank: u8,
    },
    FinishDyeing,
    SetCustomize {
        customize: CustomizeData,
    },
    ToServer(ToServer),
}
