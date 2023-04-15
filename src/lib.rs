#![feature(pointer_byte_offsets)]
use std::thread;
use std::mem::size_of;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::fs::File;
use std::path::Path;
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct Database<T>(Arc<RwLock<Inner<T>>>);

#[cfg(feature = "bincode")]
impl<T: Serialize + DeserializeOwned + Default + Send + Sync + 'static> Database<T> {
  pub fn new<P: AsRef<Path> + Clone + Send + 'static>(path: P) -> Result<Self, bincode::Error> {
    Ok(Self::new_custom(
      match File::open(path.clone()) {
        Ok(f) => bincode::deserialize_from(f)?,
        Err(_) => T::default(),
      },
      move |data| bincode::serialize_into(File::create(path.clone()).unwrap(), data).unwrap(),
    ))
  }
}

impl<T: Serialize + DeserializeOwned + Send + Sync + 'static> Database<T> {
  pub fn new_custom<S: Fn(&T) + Send + 'static>(data: T, save: S) -> Self {
    let db = Arc::new(RwLock::new(Inner { dirty: false, data }));
    let d = db.clone();
    thread::spawn(move || loop {
      let r = unsafe {
        &mut *UnsafeCell::<Inner<T>>::raw_get(
          Arc::as_ptr(&db)
            .byte_add(size_of::<RwLock<Inner<T>>>() - size_of::<UnsafeCell<Inner<T>>>())
            as _,
        )
      };
      if r.dirty {
        save(&r.data);
        r.dirty = false
      }
    });
    Self(d)
  }

  pub fn get(&self) -> ReadGuard<T> {
    ReadGuard(self.0.read().unwrap())
  }

  pub fn get_mut(&self) -> WriteGuard<T> {
    WriteGuard(self.0.write().unwrap())
  }
}

struct Inner<T> {
  dirty: bool,
  data: T,
}

pub struct ReadGuard<'a, T>(RwLockReadGuard<'a, Inner<T>>);

impl<T> Deref for ReadGuard<'_, T> {
  type Target = T;

  fn deref(&self) -> &T {
    &self.0.data
  }
}

impl<T> Deref for WriteGuard<'_, T> {
  type Target = T;

  fn deref(&self) -> &T {
    &self.0.data
  }
}

pub struct WriteGuard<'a, T>(RwLockWriteGuard<'a, Inner<T>>);

impl<T> DerefMut for WriteGuard<'_, T> {
  fn deref_mut(&mut self) -> &mut T {
    &mut self.0.data
  }
}

impl<T> Drop for WriteGuard<'_, T> {
  fn drop(&mut self) {
    self.0.dirty = true;
  }
}

#[cfg(test)]
mod test {
  use serde::{Serialize, Deserialize};
  use super::*;

  #[derive(Serialize, Deserialize, Default)]
  struct Test {
    a: u32,
  }

  #[test]
  fn test() {
    let db = Database::<Test>::new("test.db").unwrap();
    println!("{}", db.get().a);
    db.get_mut().a = 3;
  }
}
