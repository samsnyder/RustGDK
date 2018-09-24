use worker::schema::GeneratedSchema;

#[derive(Debug, Default)]
pub struct Entity<S: GeneratedSchema> {
    pub bit_field: S::ComponentBitField,
    pub chunk_index: usize,
    pub index_in_chunk: usize,
}

impl<S: GeneratedSchema> Entity<S> {
    pub fn new(
        chunk_index: usize,
        index_in_chunk: usize,
        bit_field: S::ComponentBitField,
    ) -> Entity<S> {
        Entity {
            bit_field,
            chunk_index,
            index_in_chunk,
        }
    }
}
