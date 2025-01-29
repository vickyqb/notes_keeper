use ic_cdk::{export_candid, query, update, api};
use serde::{Deserialize, Serialize};
use candid::{CandidType, Encode, Decode};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    Storable, StableBTreeMap, DefaultMemoryImpl, storable::Bound
};
use std::{borrow::Cow, cell::RefCell};
type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Serialize, Deserialize, Clone)]
struct Note {
    id: u32,
    content: String,
    owner: String,
    shared_with: Vec<String>,
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
fn get_notes() -> Vec<Note> {
    let caller = api::caller().to_string();
    NOTES.with(|notes| {
        notes.borrow().iter()
            .map(|(_, note)| note.clone())
            .filter(|note| note.owner == caller || note.shared_with.contains(&caller))
            .collect()
    })
}

#[query]
fn get_note_by_id(id: u32) -> Option<Note> {
    let caller = api::caller().to_string();
    NOTES.with(|notes| {
        notes.borrow().get(&id).filter(|note| note.owner == caller || note.shared_with.contains(&caller))
    })
}

#[update]
fn add_note(content: String) {
    let note = Note {
        id: NOTES.with(|notes| notes.borrow().len() as u32),
        content,
        owner: api::caller().to_string(),
        shared_with: Vec::new(),
    };
    NOTES.with(|notes| {
        notes.borrow_mut().insert(note.id, note);
    });
}

#[update]
fn update_note(id: u32, content: String) -> Result<String, String> {
    let caller = api::caller().to_string();
    NOTES.with(|notes| {
        let mut notes = notes.borrow_mut();
        if let Some(mut note) = notes.get(&id) {
            if note.owner == caller {
                note.content = content;
                notes.insert(id, note);
                Ok("Note updated successfully".to_string())
            } else {
                Err("Permission denied: Only the owner can update the note".to_string())
            }
        } else {
            Err("Note not found".to_string())
        }
    })
}

#[update]
fn delete_note(id: u32) -> Result<String, String> {
    let caller = api::caller().to_string();
    NOTES.with(|notes| {
        let mut notes = notes.borrow_mut();
        if let Some(note) = notes.get(&id) {
            if note.owner == caller {
                notes.remove(&id);
                Ok("Note has been deleted.".to_string())
            } else {
                Err("Permission denied: Only the owner can delete the note".to_string())
            }
        } else {
            Err("No note found.".to_string())
        }
    })
}

#[update]
fn share_note(id: u32, user: String) -> Result<String, String> {
    let caller = api::caller().to_string();
    NOTES.with(|notes| {
        let mut notes = notes.borrow_mut();
        if let Some(mut note) = notes.get(&id) {
            if note.owner == caller {
                if !note.shared_with.contains(&user) {
                    note.shared_with.push(user);
                    notes.insert(id, note);
                    Ok("Note shared successfully.".to_string())
                } else {
                    Err("User already has access to this note.".to_string())
                }
            } else {
                Err("Permission denied: Only the owner can share the note.".to_string())
            }
        } else {
            Err("Note not found.".to_string())
        }
    })
}

#[query]
fn get_notes_by_owner(owner: String) -> Vec<Note> {
    NOTES.with(|notes| {
        notes.borrow()
            .iter()
            .map(|(_, note)| note.clone())
            .filter(|note| note.owner == owner)
            .collect()
    })
}

#[query]
fn get_pid() -> String {
    return api::caller().to_string();
}

export_candid!();
