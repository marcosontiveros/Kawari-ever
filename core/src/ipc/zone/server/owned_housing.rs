use crate::common::LandData;
use binrw::binrw;

#[binrw]
#[derive(Clone, Copy, Debug, Default)]
pub struct OwnedHousing {
    /// Your free company estate.
    #[brw(pad_after = 12)] // believe these are always empty?
    pub free_company: LandData,
    /// Your personal estate.
    #[brw(pad_after = 12)]
    pub personal: LandData,
    /// Your free company private chambers.
    #[brw(pad_after = 12)]
    pub private_chambers: LandData, // Inherits ESTATE_HALL_BUILT, TELEPORTABLE and IS_IN_SUBDIVISION from free_company_estate
    /// Your first shared estate.
    #[brw(pad_after = 4)]
    pub first_shared: LandData,
    /// Your second shared estate.
    #[brw(pad_after = 12)]
    pub second_shared: LandData,
    /// Your apartment unit.
    #[brw(pad_after = 12)]
    pub apartment: LandData,
}
