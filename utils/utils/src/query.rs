/// `query!(container, index, mut field, field)`
/// - `container`: Something that implement `raw_table()` returning a `&mut RawTable`.
/// - `index`: The index (usize) to query from the container.
/// - `mut field`: 0 or more literal mut followed by a field to query. Will return a mutable reference.
/// - `field`: 0 or more field to query. Will return reference.
///
/// # Example:
/// ```ignore
/// #[derive(Fields, Columns, Components)]
/// struct Data {
///     position: (f32, f32),
///     velocity: (f32, f32),
/// }
///
/// let mut soa = Soa::with_capacity(512);
/// // Fill the map with Data struct...
///
/// for i in 0..soa.len() {
///     let (
///         position: &mut (f32, f32),
///         velocity: &(f32, f32),
///     ) = query!(soa, i, mut Data::position, Data::velocity);
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! query {
    ( $container:expr, $index:expr, $( mut $fieldmut:expr ),* , $( $field:expr ),* $(,)? ) => {
        unsafe {
            (
                $( &mut *$container.raw_table().ptr($fieldmut).add($index), )*
                $( &*$container.raw_table().ptr($field).add($index), )*
            )
        }
    };
    ( $container:expr, $index:expr, $( mut $fieldmut:expr ),* $(,)? ) => {
        unsafe {
            (
                $( &mut *$container.raw_table().ptr($fieldmut).add($index), )*
            )
        }
    };
    ( $container:expr, $index:expr, $( $field:expr ),* $(,)? ) => {
        unsafe {
            (
                $( &*$container.raw_table().ptr($field).add($index), )*
            )
        }
    };
}

/// `query!(container, field)`
/// - `container`: Something that implement `raw_table()` returning a `&mut RawTable`.
/// - `field`: 0 or more field to query. Will return a pointer to the first value of the raw table.
///
/// Faster for doing iteration than `query!`.
///
/// # Example:
/// ```ignore
/// #[derive(Fields, Columns, Components)]
/// struct Data {
///     position: (f32, f32),
///     velocity: (f32, f32),
/// }
///
/// let mut soa = Soa::with_capacity(512);
/// // Fill the map with Data struct...
///
///     let (
///         position_ptr: *mut (f32, f32),
///         velocity_ptr: *mut (f32, f32),
///     ) = query!(soa, Data::position, Data::velocity);
///
/// for i in 0..soa.len() {
///     let (
///         position: &mut (f32, f32),
///         velocity: &(f32, f32),
///     ) = unsafe {(
///         &mut *position_ptr.add(i),
///         & *velocity_ptr.add(i),
///     )};
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! query_ptr {
    ( $container:expr, $( $field:expr ),* $(,)? ) => {
        (
            $( $container.raw_table().ptr($field), )*
        )
    };
}
