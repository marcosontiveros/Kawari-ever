use crate::common::LandData;
use binrw::binrw;

#[binrw]
#[derive(Clone, Copy, Debug, Default)]
pub struct OwnedHousing {
    /// Your free company estate.
    #[brw(pad_after = 8)] // believe these are always empty?
    pub free_company: LandData,
    /// Your personal estate.
    #[brw(pad_after = 8)]
    pub personal: LandData,
    /// Unknown.
    #[brw(pad_after = 8)]
    pub unk1: LandData,
    /// Your first shared estate.
    pub first_shared: LandData,
    /// Your second shared estate.
    #[brw(pad_after = 8)]
    pub second_shared: LandData,
    /// Your apartment unit.
    #[brw(pad_after = 8)]
    pub apartment: LandData,
}
