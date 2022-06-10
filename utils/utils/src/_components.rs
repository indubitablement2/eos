use crate::*;

/// Allow moving a whole struct to and from a RawTalbe.
pub unsafe trait Components: Sized + Columns {
    /// Move the elements from the struct to the table.
    unsafe fn move_to_table(self, raw_table: &mut RawTable<Self>, index: usize);
    /// Move the elements from the table to a new struct.
    ///
    /// This is useful to drop the struct's elements.
    unsafe fn move_from_table(raw_table: &mut RawTable<Self>, index: usize) -> Self;
}
