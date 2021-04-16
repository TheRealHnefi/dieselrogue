use specs::prelude::*;
use specs::saveload::{SimpleMarker, SerializeComponents, DeserializeComponents, MarkedBuilder, SimpleMarkerAllocator};
use specs::error::NoError;
use super::*;
use std::fs::File;
use std::path::Path;
use std::fs;

const SAVEGAME_PATH: &str = "./savegame.json";

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMarker>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &mut $data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocator
            &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn save_game(ecs: &mut World) -> Result<(), std::io::Error> {
    let map_copy = ecs.get_mut::<Map>().unwrap().clone();
    let save_helper = ecs.create_entity()
        .with(SerializationHelper {map: map_copy})
        .marked::<SimpleMarker<SerializeMarker>>()
        .build();

    {
        let data = (ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMarker>>());

        let writer = File::create(SAVEGAME_PATH)?;
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            ecs, serializer, data,
            SerializationHelper,
            Player,
            Enemy,
            Position,
            Size,
            Direction,
            Facing,
            Vehicle,
            Renderable,
            LargeRenderable,
            Viewshed,
            Name,
            BlocksTile,
            GettableItem,
            GettingItem,
            Inventory,
            HumanoidBody
        );
    }

    ecs.delete_entity(save_helper).expect("Can't clean up serializer");

    Ok(())
}

pub fn load_game(ecs: &mut World) -> Result<(), ()> {
    {
        let mut entities_to_delete = Vec::new();
        for entity in ecs.entities().join() {
            entities_to_delete.push(entity);
        }
        for delentity in entities_to_delete.iter() {
            match ecs.delete_entity(*delentity) {
                Err(_) => return Err(()),
                Ok(_) => ()
            }
        }
    }

    let json_data = match fs::read_to_string(SAVEGAME_PATH) {
        Ok(res) => res,
        Err(_) => return Err(())
    };
    let mut deserializer = serde_json::Deserializer::from_str(&json_data);
    {
        let mut data = (&mut ecs.entities(),
                        &mut ecs.write_storage::<SimpleMarker<SerializeMarker>>(),
                        &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMarker>>());
        deserialize_individually!(
            ecs, deserializer, data,
            SerializationHelper,
            Player,
            Enemy,
            Position,
            Size,
            Direction,
            Facing,
            Vehicle,
            Renderable,
            LargeRenderable,
            Viewshed,
            Name,
            BlocksTile,
            GettableItem,
            GettingItem,
            Inventory,
            HumanoidBody
        );
    }

    let mut helper_entity: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let player = ecs.read_storage::<Player>();
        
        for (entity, help) in (&entities, &helper).join() {
            let mut map = ecs.write_resource::<Map>();
            *map = help.map.clone();
            map.clear_contents_index();
            helper_entity = Some(entity);
        }
        for (entity, _player) in (&entities, &player).join() {
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = entity;
        }
    }

    match ecs.delete_entity(helper_entity.unwrap()) {
        Ok(_) => Ok(()),
        Err(_) => return Err(())
    }
}
