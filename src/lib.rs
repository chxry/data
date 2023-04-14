use std::thread;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::ops::{Deref, DerefMut};
use std::fs::File;
use std::error::Error;
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct Database<T>(Arc<RwLock<Inner<T>>>);

impl<T: Serialize + DeserializeOwned + Default + Send + Sync + 'static> Database<T> {
  pub fn new(path: String) -> Result<Self, Box<dyn Error>> {
    let db = Arc::new(RwLock::new(Inner {
      dirty: false,
      data: match File::open(path.clone()) {
        Ok(f) => bincode::deserialize_from(f)?,
        Err(_) => T::default(),
      },
    }));
    let d = db.clone();
    thread::spawn(move || loop {
      let r = db.read().unwrap();
      if r.dirty {
        bincode::serialize_into(File::create(path.clone()).unwrap(), &r.data).unwrap();
        db.write().unwrap().dirty = false;
      }
    });
    Ok(Self(d))
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
    let db = Database::<Test>::new("test.db".to_string()).unwrap();
    println!("{}", db.get().a);
    db.get_mut().a = 3;
  }
}
