pub use std::sync::{Arc, RwLock};
use std::{
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
    sync::{Mutex, RwLockReadGuard, RwLockWriteGuard},
};

use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub struct ObjectBuffer<T> {
    pub buffer: Box<[RwLock<ObjectSlot<T>>]>,
    pub free_queue: Mutex<Vec<(u64, u64)>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectSlot<T> {
    pub value: Option<T>,
    pub generation: u64,
}

#[derive(Clone, Copy)]
pub struct ObjectRef<'a, T> {
    idx: u64,
    generation: u64,
    parent: Option<&'a ObjectBuffer<T>>,
}

pub struct ObjectRead<'a, T> {
    guard: RwLockReadGuard<'a, ObjectSlot<T>>,
}

pub struct ObjectWrite<'a, T> {
    guard: RwLockWriteGuard<'a, ObjectSlot<T>>,
}

pub struct ObjectIterator<'a, T> {
    parent: &'a ObjectBuffer<T>,
    idx: u64,
}

pub struct ObjectIteratorMut<'a, T> {
    parent: &'a ObjectBuffer<T>,
    idx: u64,
}

pub struct ObjectIteratorNonBlocking<'a, T> {
    parent: &'a ObjectBuffer<T>,
    idx: u64,
}

pub struct ObjectIteratorMutNonBlocking<'a, T> {
    parent: &'a ObjectBuffer<T>,
    idx: u64,
}

pub struct EventBuffer<T> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

pub struct EventBufferPull<T> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

#[derive(Clone, Copy)]
pub struct EntitySlot {
    pub is_valid: bool,
    pub generation: u64,
}
pub struct ECS {
    pub entities: Box<[RwLock<EntitySlot>]>,
    pub components: RwLock<Vec<&'static dyn TObjectBuffer>>,
}

#[derive(Clone, Copy)]
pub struct EntityId {
    idx: u64,
    generation: u64,
}

pub enum CallbackResult<T, U> {
    Pending,
    NextStage(Callback<T, U>),
    Finished(U),
}

pub struct Callback<T, U> {
    cb: Box<dyn FnMut(&T) -> CallbackResult<T, U> + Send + Sync>,
}

pub trait TObjectBuffer: Send + Sync {
    fn delete_object(&self, at: u64, generation: u64);
    fn run_delete_queue(&self);
    fn object_capacity(&self) -> usize;
    fn save(&self) -> HashMap<u64, Vec<u8>>;
    fn load(&self, x: HashMap<u64, Vec<u8>>);
}

impl<T> Default for ObjectSlot<T> {
    fn default() -> Self {
        Self {
            value: None,
            generation: 0,
        }
    }
}
impl<'a, T> Default for ObjectRef<'a, T> {
    fn default() -> Self {
        Self {
            idx: 0,
            generation: 0,
            parent: None,
        }
    }
}

impl<T> ObjectBuffer<T> {
    pub fn new() -> Self {
        let mut buf = Vec::new();
        buf.reserve_exact(32000);
        for _ in 0..32000 {
            buf.push(RwLock::new(ObjectSlot::default()))
        }
        Self {
            buffer: buf.into(),
            free_queue: Mutex::new(Vec::new()),
        }
    }

    pub fn new_capacity(capacity: usize) -> Self {
        let mut buf = Vec::new();
        buf.reserve_exact(capacity);
        for _ in 0..capacity {
            buf.push(RwLock::new(ObjectSlot::default()))
        }
        Self {
            buffer: buf.into(),
            free_queue: Mutex::new(Vec::new()),
        }
    }

    pub fn allocate_object<'a>(&'a self, item: T) -> Result<ObjectRef<'a, T>, T> {
        for i in 1..self.buffer.len() {
            let Ok(mut tmp) = self.buffer[i].try_write() else {
                continue;
            };
            if tmp.value.is_some() {
                continue;
            }
            tmp.generation = tmp.generation.wrapping_add(1);
            tmp.value = Some(item);
            return Ok(ObjectRef {
                idx: i as u64,
                generation: tmp.generation,
                parent: Some(self),
            });
        }
        Err(item)
    }

    pub fn get_object<'a, 'b>(&'a self, rf: &'b ObjectRef<'a, T>) -> Option<ObjectRead<'a, T>> {
        if let Some(p) = rf.parent {
            if p as *const _ != self as *const _ {
                return None;
            } else {
                if rf.idx as usize >= self.buffer.len() {
                    return None;
                }
                let tmp = match self.buffer[rf.idx as usize].read() {
                    Ok(p) => p,
                    Err(p) => p.into_inner(),
                };
                if tmp.generation != rf.generation {
                    None
                } else if tmp.value.is_none() {
                    None
                } else {
                    Some(ObjectRead { guard: tmp })
                }
            }
        } else {
            return None;
        }
    }

    pub fn get_object_mut<'a, 'b>(
        &'a self,
        rf: &'b ObjectRef<'a, T>,
    ) -> Option<ObjectWrite<'a, T>> {
        if let Some(p) = rf.parent {
            if p as *const _ != self as *const _ {
                return None;
            } else {
                if rf.idx as usize >= self.buffer.len() {
                    return None;
                }
                let tmp = match self.buffer[rf.idx as usize].write() {
                    Ok(p) => p,
                    Err(p) => p.into_inner(),
                };
                if tmp.generation != rf.generation {
                    None
                } else if tmp.value.is_none() {
                    None
                } else {
                    Some(ObjectWrite { guard: tmp })
                }
            }
        } else {
            return None;
        }
    }

    pub fn try_get_object<'a, 'b>(&'a self, rf: &'b ObjectRef<'a, T>) -> Option<ObjectRead<'a, T>> {
        if let Some(p) = rf.parent {
            if p as *const _ != self as *const _ {
                return None;
            } else {
                if rf.idx as usize >= self.buffer.len() {
                    return None;
                }
                let tmp = match self.buffer[rf.idx as usize].try_read() {
                    Ok(p) => p,
                    Err(p) => match p {
                        std::sync::TryLockError::Poisoned(p) => p.into_inner(),
                        std::sync::TryLockError::WouldBlock => {
                            return None;
                        }
                    },
                };
                if tmp.generation != rf.generation {
                    None
                } else if tmp.value.is_none() {
                    None
                } else {
                    Some(ObjectRead { guard: tmp })
                }
            }
        } else {
            return None;
        }
    }

    pub fn try_get_object_mut<'a, 'b>(
        &'a self,
        rf: &'b ObjectRef<'a, T>,
    ) -> Option<ObjectWrite<'a, T>> {
        if let Some(p) = rf.parent {
            if p as *const _ != self as *const _ {
                return None;
            } else {
                if rf.idx as usize >= self.buffer.len() {
                    return None;
                }
                let tmp = match self.buffer[rf.idx as usize].try_write() {
                    Ok(p) => p,
                    Err(p) => match p {
                        std::sync::TryLockError::Poisoned(p) => p.into_inner(),
                        std::sync::TryLockError::WouldBlock => {
                            return None;
                        }
                    },
                };
                if tmp.generation != rf.generation {
                    None
                } else if tmp.value.is_none() {
                    None
                } else {
                    Some(ObjectWrite { guard: tmp })
                }
            }
        } else {
            return None;
        }
    }

    pub fn deallocate_object_immediate(&self, rf: &ObjectRef<T>) {
        let Some(mut x) = self.get_object_mut(rf) else {
            return;
        };
        x.guard.value = None;
    }

    pub fn deallocate_object(&self, rf: &ObjectRef<T>) {
        let mut guard = match self.free_queue.lock() {
            Ok(p) => p,
            Err(p) => p.into_inner(),
        };
        guard.push((rf.generation, rf.idx))
    }

    pub fn run_free_queue(&self) {
        let mut guard = match self.free_queue.lock() {
            Ok(p) => p,
            Err(p) => p.into_inner(),
        };
        for i in guard.iter() {
            let rf = ObjectRef {
                idx: i.0,
                generation: i.1,
                parent: Some(self),
            };
            self.deallocate_object_immediate(&rf);
        }
        guard.clear();
    }

    pub fn unsafe_allocate_object_at<'a>(
        &'a self,
        idx: u64,
        generation: u64,
        value: T,
    ) -> Option<ObjectRef<'a, T>> {
        if idx as usize > self.buffer.len() {
            return None;
        }
        let mut guard = match self.buffer[idx as usize].write() {
            Ok(p) => p,
            Err(p) => p.into_inner(),
        };
        guard.value = Some(value);
        guard.generation = generation;
        Some(ObjectRef {
            idx,
            generation,
            parent: Some(self),
        })
    }

    pub fn unsafe_construct_reference<'a>(&'a self, idx: u64, generation: u64) -> ObjectRef<'a, T> {
        ObjectRef {
            idx,
            generation,
            parent: Some(self),
        }
    }

    pub fn iter<'a>(&'a self) -> ObjectIterator<'a, T> {
        ObjectIterator {
            parent: self,
            idx: 0,
        }
    }

    pub fn iter_non_blocking<'a>(&'a self) -> ObjectIteratorNonBlocking<'a, T> {
        ObjectIteratorNonBlocking {
            parent: self,
            idx: 0,
        }
    }

    pub fn iter_mut<'a>(&'a self) -> ObjectIteratorMut<'a, T> {
        ObjectIteratorMut {
            parent: self,
            idx: 0,
        }
    }

    pub fn iter_mut_non_blocking<'a>(&'a self) -> ObjectIteratorMutNonBlocking<'a, T> {
        ObjectIteratorMutNonBlocking {
            parent: self,
            idx: 0,
        }
    }

    pub fn for_each(&self, to_run: impl Fn(ObjectRead<'_, T>)) {
        for i in self.iter() {
            to_run(i)
        }
    }

    pub fn for_each_mut(&self, to_run: impl Fn(ObjectWrite<'_, T>)) {
        for i in self.iter_mut() {
            to_run(i)
        }
    }
}

impl<T: Send + Sync> ObjectBuffer<T> {
    pub fn for_each_par(&self, to_run: impl Fn(ObjectRef<T>, &T) + Sync) {
        let l = self.buffer.len();
        (0..l).par_bridge().for_each(|i| {
            let lck = match self.buffer[i].read() {
                Ok(p) => p,
                Err(p) => p.into_inner(),
            };
            let genr = lck.generation;
            let rf = ObjectRef {
                idx: i as u64,
                generation: genr,
                parent: Some(self),
            };
            let g = ObjectRead { guard: lck };
            to_run(rf, &*g);
        });
    }

    pub fn for_each_mut_par(&self, to_run: impl Fn(ObjectRef<T>, &mut T) + Sync) {
        let l = self.buffer.len();
        (0..l).par_bridge().for_each(|i| {
            let lck = match self.buffer[i].write() {
                Ok(p) => p,
                Err(p) => p.into_inner(),
            };
            let genr = lck.generation;
            let rf = ObjectRef {
                idx: i as u64,
                generation: genr,
                parent: Some(self),
            };
            let mut g = ObjectWrite { guard: lck };
            to_run(rf, &mut *g);
        });
    }

    pub fn for_each_par_no_block(&self, to_run: impl Fn(ObjectRef<T>, &T) + Sync) {
        let l = self.buffer.len();
        (0..l).par_bridge().for_each(|i| {
            let lck = match self.buffer[i].try_read() {
                Ok(p) => p,
                Err(p) => match p {
                    std::sync::TryLockError::WouldBlock => {
                        return;
                    }
                    std::sync::TryLockError::Poisoned(p) => p.into_inner(),
                },
            };
            let genr = lck.generation;
            let rf = ObjectRef {
                idx: i as u64,
                generation: genr,
                parent: Some(self),
            };
            let g = ObjectRead { guard: lck };
            to_run(rf, &*g);
        });
    }

    pub fn for_each_par_mut_no_block(&self, to_run: impl Fn(ObjectRef<T>, &mut T) + Sync) {
        let l = self.buffer.len();
        (0..l).par_bridge().for_each(|i| {
            let lck = match self.buffer[i].try_write() {
                Ok(p) => p,
                Err(p) => match p {
                    std::sync::TryLockError::WouldBlock => {
                        return;
                    }
                    std::sync::TryLockError::Poisoned(p) => p.into_inner(),
                },
            };
            let genr = lck.generation;
            let mut g = ObjectWrite { guard: lck };
            to_run(
                ObjectRef {
                    generation: genr,
                    idx: i as u64,
                    parent: Some(&self),
                },
                &mut *g,
            );
        });
    }
}

impl<T: Send + Sync + Clone> ObjectBuffer<T> {
    pub fn update(&self, to_run: impl Fn(ObjectRef<T>, &mut T) + Sync) {
        let l = self.buffer.len();
        let output: Vec<ObjectSlot<T>> = (0..l)
            .par_bridge()
            .map(|i| {
                let lck = match self.buffer[i].read() {
                    Ok(p) => p,
                    Err(p) => p.into_inner(),
                };
                let mut output = lck.clone();
                drop(lck);
                if let Some(x) = output.value.as_mut() {
                    to_run(
                        ObjectRef {
                            idx: i as u64,
                            generation: output.generation,
                            parent: Some(self),
                        },
                        x,
                    );
                }
                output
            })
            .collect();
        for (idx, i) in output.into_iter().enumerate() {
            let mut guard = match self.buffer[idx].write() {
                Ok(p) => p,
                Err(p) => p.into_inner(),
            };
            *guard = i;
        }
    }
}

impl<'a, T> Deref for ObjectRead<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.value.as_ref().unwrap()
    }
}

impl<'a, T> Deref for ObjectWrite<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.value.as_ref().unwrap()
    }
}

impl<'a, T> DerefMut for ObjectWrite<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.value.as_mut().unwrap()
    }
}

impl<'a, T> ObjectRef<'a, T> {
    pub fn is_valid(&self) -> bool {
        self.idx != 0 && self.parent.is_some()
    }

    pub const fn new_invalid() -> Self {
        Self {
            idx: 0,
            generation: 0,
            parent: None,
        }
    }

    pub fn try_get(&self) -> Option<ObjectRead<'a, T>> {
        if let Some(x) = self.parent.as_ref() {
            x.try_get_object(self)
        } else {
            None
        }
    }

    pub fn try_get_mut(&self) -> Option<ObjectWrite<'a, T>> {
        if let Some(x) = self.parent.as_ref() {
            x.try_get_object_mut(self)
        } else {
            None
        }
    }

    pub fn get(&self) -> Option<ObjectRead<'a, T>> {
        if let Some(x) = self.parent.as_ref() {
            x.get_object(self)
        } else {
            None
        }
    }

    pub fn get_mut(&self) -> Option<ObjectWrite<'a, T>> {
        if let Some(x) = self.parent.as_ref() {
            x.get_object_mut(self)
        } else {
            None
        }
    }

    pub fn get_panic(&self) -> ObjectRead<'a, T> {
        if let Some(x) = self.parent.as_ref() {
            x.get_object(self).unwrap()
        } else {
            todo!()
        }
    }

    pub fn get_mut_panic(&self) -> ObjectWrite<'a, T> {
        if let Some(x) = self.parent.as_ref() {
            x.get_object_mut(self).unwrap()
        } else {
            todo!()
        }
    }

    pub fn delete(&self) {
        if let Some(x) = self.parent.as_ref() {
            x.deallocate_object(self);
        } else {
            return;
        }
    }

    pub fn delete_now(&self) {
        if let Some(x) = self.parent.as_ref() {
            x.deallocate_object_immediate(self);
        } else {
            return;
        }
    }
}

impl<'a, T> Iterator for ObjectIterator<'a, T> {
    type Item = ObjectRead<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.parent.buffer.len() as u64 {
            let guard = match self.parent.buffer[self.idx as usize].read() {
                Ok(p) => p,
                Err(p) => p.into_inner(),
            };
            self.idx += 1;
            if guard.value.is_some() {
                return Some(ObjectRead { guard: guard });
            }
        }
        None
    }
}
impl<'a, T> Iterator for ObjectIteratorNonBlocking<'a, T> {
    type Item = ObjectRead<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.parent.buffer.len() as u64 {
            let guard = match self.parent.buffer[self.idx as usize].try_read() {
                Ok(p) => p,
                Err(p) => match p {
                    std::sync::TryLockError::Poisoned(p) => p.into_inner(),
                    std::sync::TryLockError::WouldBlock => {
                        self.idx += 1;
                        continue;
                    }
                },
            };
            self.idx += 1;
            if guard.value.is_some() {
                return Some(ObjectRead { guard: guard });
            }
        }
        None
    }
}

impl<'a, T> Iterator for ObjectIteratorMut<'a, T> {
    type Item = ObjectWrite<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.parent.buffer.len() as u64 {
            let guard = match self.parent.buffer[self.idx as usize].write() {
                Ok(p) => p,
                Err(p) => p.into_inner(),
            };
            self.idx += 1;
            if guard.value.is_some() {
                return Some(ObjectWrite { guard: guard });
            }
        }
        None
    }
}

impl<'a, T> Iterator for ObjectIteratorMutNonBlocking<'a, T> {
    type Item = ObjectWrite<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.parent.buffer.len() as u64 {
            let guard = match self.parent.buffer[self.idx as usize].try_write() {
                Ok(p) => p,
                Err(p) => match p {
                    std::sync::TryLockError::Poisoned(p) => p.into_inner(),
                    std::sync::TryLockError::WouldBlock => {
                        self.idx += 1;
                        continue;
                    }
                },
            };
            self.idx += 1;
            if guard.value.is_some() {
                return Some(ObjectWrite { guard: guard });
            }
        }
        None
    }
}

impl<T> Clone for EventBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Iterator for EventBufferPull<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let mut list = match self.inner.lock() {
            Ok(p) => p,
            Err(p) => p.into_inner(),
        };
        list.pop_front()
    }
}

impl<T> EventBuffer<T> {
    pub fn new_event(&self, ev: T) {
        let mut list = match self.inner.lock() {
            Ok(p) => p,
            Err(p) => p.into_inner(),
        };
        list.push_back(ev);
    }

    pub fn pull(&self) -> EventBufferPull<T> {
        EventBufferPull {
            inner: self.inner.clone(),
        }
    }
}

impl EventBuffer<Box<dyn FnOnce()>> {
    pub fn update(&self) {
        for i in self.pull() {
            (i)();
        }
    }
}

impl EventBuffer<Box<dyn FnMut()>> {
    pub fn update(&self) {
        for mut i in self.pull() {
            (i)();
        }
    }
}
impl EventBuffer<Box<dyn Fn()>> {
    pub fn update(&self) {
        for i in self.pull() {
            (i)();
        }
    }
}
impl<T> EventBuffer<Box<dyn FnOnce(&T)>> {
    pub fn update(&self, context: &T) {
        for i in self.pull() {
            (i)(context);
        }
    }
}

impl<T> EventBuffer<Box<dyn FnMut(&T)>> {
    pub fn update(&self, context: &T) {
        for mut i in self.pull() {
            (i)(context);
        }
    }
}

impl<T> EventBuffer<Box<dyn Fn(&T)>> {
    pub fn update(&self, context: &T) {
        for i in self.pull() {
            (i)(context);
        }
    }
}

impl<T, U> EventBuffer<Callback<T, U>> {
    pub fn drive(&self, context: &T) -> Vec<U> {
        let mut out = Vec::new();
        for mut i in self.pull() {
            match i.update(context) {
                CallbackResult::Finished(x) => out.push(x),
                CallbackResult::NextStage(s) => {
                    self.new_event(s);
                }
                CallbackResult::Pending => {
                    self.new_event(i);
                }
            }
        }
        out
    }
}

impl<T: Send + Sync, U: Send + Sync> EventBuffer<Callback<T, U>> {
    pub fn drive_par(&self, context: &T) -> Vec<U> {
        let out: Vec<Option<U>> = self
            .pull()
            .par_bridge()
            .map(|mut i| {
                match i.update(context) {
                    CallbackResult::Finished(x) => {
                        return Some(x);
                    }
                    CallbackResult::NextStage(s) => {
                        self.new_event(s);
                    }
                    CallbackResult::Pending => {
                        self.new_event(i);
                    }
                }
                None
            })
            .collect();
        out.into_iter()
            .filter(|i| i.is_some())
            .map(|i| i.unwrap())
            .collect()
    }

    pub fn drive_par_discard(&self, context: &T) {
        self.pull()
            .par_bridge()
            .for_each(|mut i| match i.update(context) {
                CallbackResult::Finished(x) => {
                    _ = x;
                }
                CallbackResult::NextStage(s) => {
                    self.new_event(s);
                }
                CallbackResult::Pending => {
                    self.new_event(i);
                }
            });
    }
}

impl<T, U> Callback<T, U> {
    pub fn update(&mut self, context: &T) -> CallbackResult<T, U> {
        (self.cb)(context)
    }
}

#[macro_export]
macro_rules! poll {
    ($to_poll:ident, $context:ident) => {
        loop {
            match ($to_poll.update($context)) {
                CallbackResult::Done(x) => break x,
                CallbackResult::Pending => {
                    return CallbackResult::Pending;
                }
                CallbackResult::NextStage(v) => {
                    ($to_poll) = v;
                }
            }
        }
    };
}

impl<T: Serialize + DeserializeOwned + Send + Sync> TObjectBuffer for ObjectBuffer<T> {
    fn delete_object(&self, at: u64, generation: u64) {
        self.deallocate_object(&ObjectRef {
            idx: at,
            generation,
            parent: Some(self),
        });
    }

    fn run_delete_queue(&self) {
        self.run_free_queue();
    }

    fn object_capacity(&self) -> usize {
        self.buffer.len()
    }

    fn load(&self, from: HashMap<u64, Vec<u8>>) {
        for i in 0..self.buffer.iter().len() {
            if let Some(n) = from.get(&(i as u64)) {
                let mut guard = match self.buffer[i].write() {
                    Ok(p) => p,
                    Err(p) => p.into_inner(),
                };
                *guard = rmp_serde::from_slice(&*n).unwrap();
            } else {
                let mut guard = match self.buffer[i].write() {
                    Ok(p) => p,
                    Err(p) => p.into_inner(),
                };
                *guard = ObjectSlot {
                    value: None,
                    generation: 0,
                };
            }
        }
    }

    fn save(&self) -> HashMap<u64, Vec<u8>> {
        let mut out = HashMap::new();
        for i in 0..self.buffer.len() {
            let guard = match self.buffer[i].read() {
                Ok(p) => p,
                Err(p) => p.into_inner(),
            };
            out.insert(i as u64, rmp_serde::to_vec(&*guard).unwrap());
        }
        out
    }
}

impl ECS {
    pub fn new(comps: &[&'static dyn TObjectBuffer]) -> Self {
        let mut v = Vec::new();
        v.reserve_exact(32000);
        for _ in 0..32000 {
            v.push(RwLock::new(EntitySlot {
                is_valid: false,
                generation: 0,
            }))
        }
        Self {
            entities: v.into_boxed_slice(),
            components: RwLock::new(comps.to_vec()),
        }
    }

    pub fn alloc_entity(&self) -> Option<EntityId> {
        for j in 1..self.entities.len() {
            let i = &self.entities[j];
            let mut guard = match i.write() {
                Ok(x) => x,
                Err(e) => e.into_inner(),
            };
            if !guard.is_valid {
                guard.generation = guard.generation.wrapping_add(1);
                return Some({
                    EntityId {
                        idx: j as u64,
                        generation: guard.generation,
                    }
                });
            }
        }
        None
    }

    pub fn dealloc_entity(&self, id: EntityId) {
        if id.idx as usize >= self.entities.len() {
            return;
        }
        let mut guard = match self.entities[id.idx as usize].write() {
            Ok(p) => p,
            Err(p) => p.into_inner(),
        };
        if guard.is_valid && id.generation == guard.generation {
            guard.is_valid = false;
            let comps = match self.components.read() {
                Ok(p) => p,
                Err(p) => p.into_inner(),
            };
            for i in comps.iter() {
                i.delete_object(id.idx, id.generation);
            }
        }
    }
    pub fn is_entity_valid(&self, id: EntityId) -> bool {
        if id.idx as usize >= self.entities.len() {
            return false;
        }
        if id.idx == 0 {
            return false;
        }
        let guard = match self.entities[id.idx as usize].read() {
            Ok(p) => p,
            Err(p) => p.into_inner(),
        };
        guard.generation == id.generation && guard.is_valid
    }
}
impl EntityId {
    pub const fn new_invalid() -> Self {
        Self {
            idx: 0,
            generation: 0,
        }
    }
    pub const fn get_idx(&self) -> u64 {
        self.idx
    }
    pub const fn get_gen(&self) -> u64 {
        self.generation
    }
}

#[macro_export]
macro_rules! make_ecs {
    ($ecs_name:ident,$((
        $buffer_name:ident,
        $comp_name:ident, $comp_type:ty, $adder_name:ident,
        $remover_name:ident, $getter_name:ident,$mut_getter_name:ident,$reference_getter_name:ident
    )),*) => {
        lazy_static::lazy_static!{
            pub static ref $ecs_name:ECS = ECS::new(&[$(
                    &*$buffer_name,
            )*]);
        }
        $(lazy_static::lazy_static!{
                pub static ref $buffer_name:ObjectBuffer<$comp_type> = ObjectBuffer::new();
        })*
        #[derive(Clone,Copy)]
        pub enum ComponentType{
            $(
                $comp_name
            )*
        }
        #[derive(Clone, Copy)]
        pub struct Entity {
            id: EntityId,
        }
        pub fn new_entity()->Option<Entity>{
            Some(Entity{id:$ecs_name.alloc_entity()?})
        }
        pub fn delete_entity(entity:Entity){
            $ecs_name.dealloc_entity(entity.id);
        }
        impl Entity{
            pub const fn new_invalid()->Self{
                Self{id:EntityId::new_invalid()}
            }
            pub const fn get_idx(&self)->u64{
                self.id.get_idx()
            }
            pub const fn get_gen(&self)->u64{
                self.id.get_gen()
            }
            pub fn is_valid(&self)->bool{
                $ecs_name.is_entity_valid(self.id)
            }
            $(
                pub fn $adder_name(&self, value:$comp_type){
                    $buffer_name.unsafe_allocate_object_at(self.get_idx(), self.get_gen(), value);
                }

                pub fn $remover_name(&self){
                    let Some(object) = self.$reference_getter_name()else{
                        return;
                    };
                    $buffer_name.deallocate_object(&object);
                }

                pub fn $reference_getter_name<'a>(&'a self)->Option<ObjectRef<'a,$comp_type>>{
                    let rf = $buffer_name.unsafe_construct_reference(self.get_idx(), self.get_gen());
                    if rf.get().is_some(){
                        Some(rf)
                    }else{
                        None
                    }
                }

                pub fn $getter_name<'a>(&'a self)->Option<ObjectRead<'a,$comp_type>>{
                    self.$reference_getter_name()?.get()
                }

                pub fn $mut_getter_name<'a>(&'a self)->Option<ObjectWrite<'a,$comp_type>>{
                    self.$reference_getter_name()?.get_mut()
                }
            )*

            pub fn has_comp(&self,comp:ComponentType)->bool{
                match comp{
                    $(
                        ComponentType::$comp_name=>{
                            self.$getter_name().is_some()
                        }
                    )*
                }
            }
            pub fn has_comp_set(&self,comps:&[ComponentType])->bool{
                for i in comps{
                    if !self.has_comp(*i){
                        return false;
                    }
                }
                true
            }
        }


    };
}
