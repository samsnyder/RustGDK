use entity_template::EntityTemplate;
use std::marker::PhantomData;
use std::path::Path;
use worker::SnapshotOutputStream;
use worker::schema::GeneratedSchema;

pub struct Snapshot<S: GeneratedSchema> {
    phantom: PhantomData<S>,
}

impl<S: GeneratedSchema> Snapshot<S> {
    pub fn create<P: AsRef<Path>, I>(filename: P, entities: I)
    where
        I: Iterator<Item = EntityTemplate>,
    {
        let mut stream = SnapshotOutputStream::new(filename);
        for mut entity in entities {
            if entity.entity_id.is_none() {
                panic!("All snapshot entities must have an Entity ID set.");
            }

            let (entity_acl_id, entity_acl_data) =
                S::serialise_entity_acl(entity.read_access, entity.write_access);
            entity.data.insert(entity_acl_id, entity_acl_data);

            if let Result::Err(error) = stream.write_entity(entity.data, entity.entity_id.unwrap())
            {
                panic!("Error writing entity: {}", error);
            }
        }
    }
}
