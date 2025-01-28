use ic_cdk::{export_candid, query, update, api};
use serde::{Deserialize, Serialize};
use candid::{CandidType, Encode, Decode};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    Storable, StableBTreeMap, DefaultMemoryImpl,storable::Bound
};
use std::{borrow::Cow, cell::RefCell};
type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Serialize, Deserialize, Clone)]
struct Note {
    id: u32,
    content: String,
    owner:String
}

impl Storable for Note {
    const BOUND : Bound = Bound::Unbounded;
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default()));
    static NOTES: RefCell<StableBTreeMap<u32, Note, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}

#[query]
fn get_notes()-> Vec<Note> {
    NOTES.with(|notes| {
        notes.borrow().iter().map(|(_, note)| note.clone()).collect()
    })
}

#[query]
fn get_note_by_id(id:u32)->Option<Note>{
    NOTES.with(|notes| notes.borrow().get(&id))
}


#[update ]
fn add_note(content :String){
    let note = Note {
        id: NOTES.with(|notes| notes.borrow().len() as u32),
        content,
        owner: api::caller().to_string()
    };
    NOTES.with(|notes| {
        notes.borrow_mut().insert(note.id.clone(), note);
    });
    
}
#[update ]
fn update_note(id : u32, content :String){
    let note = Note {
        id: id,
        content,
        owner: api::caller().to_string()
    };
    NOTES.with(|notes| {
        notes.borrow_mut().insert(note.id.clone(), note);
    });
}

#[update]
fn delete_note(note_id:u32)->Result<String,String>{
    let delete_result = NOTES.with(|notes| {
        let mut notes = notes.borrow_mut();
        if notes.remove(&note_id).is_some() {
            return Ok("Note has deleted.".to_string());
        } else {
            return Err("No note found.".to_string());
        };
    });
    return delete_result;
}

#[query]
fn greet(name:String)->String{
    format!("hello {}",name)
}

#[query]
fn get_notes_by_owner(owner: String) -> Vec<Note> {
    NOTES.with(|notes| {
        notes
            .borrow()
            .iter()
            .filter_map(|(_, note)| {
                if note.owner == owner {
                    Some(note)
                } else {
                    None
                }
            })
            .collect()
    })
}

export_candid!();
