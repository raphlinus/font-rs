#[macro_export]
macro_rules! gen_new {
  ($r:ident, $($field:ident : $field_type:ty),*) => {
    impl $r {
      pub fn new($($field: $field_type),*) -> Self {
        $r {
          $($field: $field),*
        }
      }
    }
  }
}
