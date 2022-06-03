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

/// `query_ptr!(container, field)`
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
///     ) = query_ptr!(soa, Data::position, Data::velocity);
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

/// Needs `#![feature(ptr_const_cast)]`
/// 
/// `query_slice!(container, field)`
/// - `container`: Something that implement `raw_table()` returning a `&mut RawTable` and `len()`.
/// - `field`: 0 or more field to query. Will return a pointer to the first value of the raw table.
///
/// Faster for doing iteration than `query!`.
#[macro_export]
macro_rules! query_slice {
    ( $container:expr, $( mut $fieldmut:expr ),* , $( $field:expr ),* $(,)? ) => {
        unsafe {(
            $( std::slice::from_raw_parts_mut($container.raw_table().ptr($fieldmut), $container.len()) , )*
            $( std::slice::from_raw_parts($container.raw_table().ptr($field).as_const(), $container.len()) , )* 
        )}
    };
    ( $container:expr, $( mut $fieldmut:expr ),* $(,)? ) => {
        unsafe {(
            $( std::slice::from_raw_parts_mut($container.raw_table().ptr($fieldmut), $container.len()) , )*
        )}
    };
    ( $container:expr, $( $field:expr ),* $(,)? ) => {
        unsafe {(
            $( std::slice::from_raw_parts($container.raw_table().ptr($field).as_const(), $container.len()) , )*
        )}
    };
}

/// Needs `#![feature(macro_metavar_expr)]`
/// 
/// `query_closure!(container, closure, field)`
/// - `container`: Something that implement `raw_table()` returning a `&mut RawTable` and len().
/// - `closure`: A closure with the provided fields and starting with an index field. 
/// Return a bool to break early (true). 
/// - `field`: 0 or more field to query. Will return a pointer to the first value of the raw table.
/// 
/// Fastest iteration.
/// 
/// # Example:
/// ```ignore
/// let closure = |i: usize, pos: &mut Vec2, vel: &Vec2| {
///     *pos += *vel;
///     false
/// };
/// query_closure!($countainer, closure, $Struct::pos, $Struct::vel);
/// 
/// ```
#[macro_export]
macro_rules! query_closure {
    ( $container:expr, $closure:expr, $( $field:expr ),* $(,)? ) => {
        unsafe {
            let ptrs = query_ptr!($container, $( $field , )*);

            for i in 0..$container.len() {
                if $closure(i, $( ${ignore(field)} &mut *ptrs.${index()}.add(i), )* ) {
                    break;
                }
            }
        }
    };
}

#[test]
fn test_query() {
    use crate::*;
    #[derive(Debug, Fields, Columns)]
    struct C {
        a: f32,
        b: i32,
    }
    unsafe impl Components for C {
        unsafe fn move_to_table(self, raw_table: &mut RawTable<Self>, index: usize) {
            *raw_table.ptr(C::a).add(index) = self.a;
            *raw_table.ptr(C::b).add(index) = self.b;
        }

        unsafe fn move_from_table(raw_table: &mut RawTable<Self>, index: usize) -> Self {
            Self { 
                a: *raw_table.ptr(C::a).add(index),
                b: *raw_table.ptr(C::b).add(index),
            }
        }
    }

    let mut s: Soa<C> = Soa::with_capacity(10);
    s.push(C {
        a: 5.0,
        b: 6,
    });
    s.push(C {
        a: 8.0,
        b: 9,
    });

    let mut c = 0;
    let mut closure = |i: usize, a: &mut f32, b: &i32| {
        c += b;
        *a += *b as f32;
        println!("{} {} {}", i, a, b);
        false
    };
    query_closure!(s, closure, C::a, C::b);

    assert_eq!(c, 6 + 9);
}