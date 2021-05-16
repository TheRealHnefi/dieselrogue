use rltk::{VirtualKeyCode};
use super::*;

pub struct Menu {
    pub x: i32,
    pub y: i32,
    pub rows: Vec<MenuRow>,
    pub selected_row: usize,
    pub target: Option<Entity>
}

pub struct MenuRow {
    pub hotkey: VirtualKeyCode,
    pub text: String,
    pub action: MenuFunction
}

type MenuFunction = fn (menu: &Menu, ecs: &mut World) -> RunState;

impl Menu {
    pub fn new_main() -> Self {
        let save_row = MenuRow {
            hotkey: VirtualKeyCode::S,
            text: "(S) Save".to_string(),
            action: Menu::action_save
        };
        let load_row = MenuRow {
            hotkey: VirtualKeyCode::L,
            text: "(L) Load".to_string(),
            action: Menu::action_load
        };
        let quit_row = MenuRow {
            hotkey: VirtualKeyCode::Q,
            text: "(Q) Quit".to_string(),
            action: Menu::action_quit
        };
        let close_row = MenuRow {
            hotkey: VirtualKeyCode::C,
            text: "(C) Close Menu".to_string(),
            action: Menu::action_close
        };

        Self {
            x: 35,
            y: 20,
            rows: vec![save_row, load_row, quit_row, close_row],
            selected_row: 0,
            target: None
        }
    }

    pub fn new_target_menu(ecs: &World, x: i32, y: i32, target: Entity) -> Self {
        //let actions = player::valid_actions(ecs, target).expect("Error when finding valid actions");

        let mut new_rows = vec![];

        // for action in actions {
        //     match action {
        //         player::Action::Examine => rows.push(MenuRow {
        //             hotkey: VirtualKeyCode::E,
        //             text: "(E) Examine".to_string(),
        //             action: Menu::action_examine
        //         }),
        //         player::Action::Shoot => rows.push(MenuRow {
        //             hotkey: VirtualKeyCode::S,
        //             text: "(S) Shoot".to_string(),
        //             action: Menu::action_shoot
        //         }),
        //     }
        // }

        Menu {
            x: x + 1,
            y: y,
            rows: new_rows,
            selected_row: 0,
            target: Some(target)
        }
    }

    pub fn action_save(&self, _ecs: &mut World) -> RunState {
        return RunState::Saving;
    }

    pub fn action_load(&self, _ecs: &mut World) -> RunState {
        return RunState::Loading;
    }

    pub fn action_quit(&self, _ecs: &mut World) -> RunState {
        ::std::process::exit(0);
    }

    pub fn action_close(&self, _ecs: &mut World) -> RunState {
        return RunState::AwaitingInput;
    }

    pub fn action_examine(menu: &Menu, ecs: &mut World) -> RunState {
        // let mut game_log = ecs.fetch_mut::<GameLog>();
        // match menu.target {
        //     Some(entity) => {
        //         let names = ecs.read_storage::<Name>();
        //         match names.get(entity) {
        //             Some(name) => {
        //                 game_log.entries.push(name.value.to_string());
        //             }
        //             None => {
        //                 game_log.entries.push("Nameless entity".to_string());
        //             }
        //         }
        //     }
        //     None => {
        //         game_log.entries.push("Empty space".to_string());
        //     }
        // }
        return RunState::AwaitingInput;
    }

    pub fn action_shoot(&self, ecs: &mut World) -> RunState {
        // let mut game_log = ecs.fetch_mut::<GameLog>();
        // let target = self.target.unwrap();
        // let names = ecs.read_storage::<Name>();
        // let target_name = &names.get(target).unwrap().value;
        // game_log.entries.push(format!("Firing at {}", target_name));

        // let mut damages = ecs.write_storage::<Damage>();
        // match damages.get_mut(target) {
        //     Some(damage) => {
        //         damage.instances.push(DamageInstance {phys: 3, heat: 0, elec: 0});
        //     },
        //     None => {
        //         let new_damage = Damage { instances: vec![DamageInstance {phys: 3, heat: 0, elec: 0}]};
        //         damages.insert(target, new_damage).expect("Unable to create damage component");
        //     }
        // }

        return RunState::ExecuteTurn;
    }
}