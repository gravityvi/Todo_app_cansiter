use ic_cdk::{
    export:: candid::{ CandidType, Deserialize},
    
    query, update, pre_upgrade, storage, post_upgrade,
};

use std::mem;
use std::cell::RefCell;
use std::collections::BTreeMap;

type TaskStore = BTreeMap<u64, Task>;

// Task for a todo
#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct Task {
    pub id: u64,
    pub description: String,
}


impl Task {
    fn new(id: u64, description: String) -> Self {
        Task {
            id,
            description
        }
    }
}


//CanisterState persisted across upgrades
#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct CanisterState {
    counter: u64,
    tasks: TaskStore
}




thread_local! {
    static COUNTER: RefCell<u64> = RefCell::new(0);
    static TASKS: RefCell<TaskStore> = RefCell::default();

}


// get all task with an optional offset and limit (MAX: 10)
#[query(name = "getAll")]
fn get_all(offset: Option<usize>, limit: Option<usize>) -> Vec<Task> {

    let total_tasks = TASKS.with(|tasks| tasks.borrow().len());

    let max_limit = usize::min(10, total_tasks);

    let offset = usize::min(offset.unwrap_or(0), total_tasks);

    let limit = usize::min(limit.unwrap_or(max_limit), max_limit);


    TASKS.with(|tasks| {
        let all_taks: Vec<Task> = tasks.borrow_mut().values().map(|val| val.clone()).collect();
        let response = &all_taks[offset .. (usize::min(limit+offset, total_tasks))];
        response.to_owned()
    })
    
}

// get individual task with an id.
#[query]
fn get(id: u64) -> Task {
   TASKS.with(|tasks| {
        tasks.borrow()
        .get(&id)
        .and_then(|task| Some(task.clone())).unwrap_or_default()
    })
}


// Create a Task with a desc.
#[update]
fn create(desc: String) -> Task {
    TASKS.with(|tasks| {
        let id = COUNTER.with(|counter| {
            let mut cnt = counter.borrow_mut();
            *cnt += 1;
            return *cnt-1;
        });
        let task = Task::new(id, desc);
        tasks.borrow_mut().insert(id, task.clone());
        
        task
    })
}


// Update a task with id and desc.
#[update]
fn update(id: u64, desc: String) -> Task {
    TASKS.with(|tasks| {
        let mut task_to_update = tasks
            .borrow_mut()
            .get(&id)
            .unwrap()
            .clone();

        task_to_update.description = desc;
        tasks.borrow_mut().insert(id, task_to_update.clone());

        task_to_update
    })
}

#[update]
fn delete(id: u64) {
    TASKS.with(|tasks| {
        tasks
            .borrow_mut()
            .remove(&id);
    })
}


// Persist Tasks across upgrades.

#[pre_upgrade]
fn pre_upgrade() {
    let copied_counter = COUNTER.with(|counter| *counter.borrow());
    TASKS.with(|tasks| {
        let old_state = CanisterState {
            tasks: mem::take(&mut tasks.borrow_mut()),
            counter: copied_counter
        };
        storage::stable_save((old_state,)).unwrap();
    });
}

#[post_upgrade]
fn post_upgrade() {
    let (old_state,): (CanisterState,) = storage::stable_restore().unwrap();
    TASKS.with(|tasks| {
        COUNTER.with(|counter_ref| {
                *tasks.borrow_mut() = old_state.tasks;
                *counter_ref.borrow_mut() = old_state.counter;
        })
    });
}

