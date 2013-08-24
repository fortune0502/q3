/*
    Copyright 2013 Jesse 'Jeaye' Wilkerson
    See licensing in LICENSE file, or at:
        http://www.opensource.org/licenses/BSD-3-Clause

    File: state/director.rs
    Author: Jesse 'Jeaye' Wilkerson
    Description:
      A state stack that propogates update
      and render signals.
*/

use std::local_data;
use util::Log;

static tls_key: local_data::Key<Director> = &local_data::Key;

#[macro_escape]
#[path = "../util/log_macros.rs"]
mod log;

pub trait State
{
  fn load(&mut self);
  fn unload(&mut self)
  { }

  /* Each state must have a unique key that other
   * objects can use to reference it. */
  fn get_key(&self) -> &str;

  /*  Returns true when the event has been captured. If the event is not
      captured, it's set to the next lower state. (A state can capture
      renders, for example, as an optimization -- or updates as a sanity or
      security measure). Rinse and repeat. */
  fn update(&mut self, _delta: f32) -> bool /* dt is in terms of seconds. */
  { false }
  fn render(&mut self) -> bool
  { false }

  /* TODO: Trait inheritance with Input_Listener. */
  fn key_action(&mut self, _key: i32, _action: i32, _mods: i32) -> bool
  { false }
  fn key_char(&mut self, _ch: char) -> bool
  { false }
  fn mouse_action(&mut self, _button: i32, _action: i32, _mods: i32) -> bool
  { false }
  fn mouse_moved(&mut self, _x: f32, _y: f32) -> bool
  { false }
}

pub struct Director
{
  states: ~[@mut State],
}

impl Director
{
  pub fn create()
  {
    let director = Director
    {
      states: ~[],
    };

    /* Store the director in task-local storage. (singleton) */
    local_data::set(tls_key, director);
  }

  /* Accesses the singleton director from task-local storage. */
  pub fn get_mut<T>(handler: &fn(&mut Director) -> T) -> T
  {
    local_data::get_mut(tls_key, 
    |opt|
    {
      match opt
      {
        Some(x) => handler(&mut *x),
        None => fail!("Singleton not available: Director")
      }
    })
  }

  pub fn push(&mut self, state: @mut State)
  {
    state.load();
    self.states.push(state);
  }

  pub fn pop(&mut self)
  {
    let state = self.states.pop();
    state.unload();
  }

  /* Removes the state with the specified key. */
  pub fn pull(&mut self, key: &str)
  {
    let index = do self.states.rposition |state|
    { state.get_key() == key };
    match index
    {
      Some(i) =>
      {
        let state = self.states.remove(i);
        state.unload();
      }
      None => { log_debug!("Invalid state to pull '%s'", key); }
    }
  }

  /* Swaps the state specified by key with the newly
   * specified state. */
  pub fn swap(&mut self, key: &str, state: @mut State)
  {
    let index = do self.states.rposition |st|
    { st.get_key() == key };
    match index
    {
      Some(i) =>
      {
        let old_state = self.states[i];
        old_state.unload();

        state.load();
        self.states[i] = state;
      }
      None => { log_debug!("Invalid state to swap '%s'", key); }
    }
  }

  /* Unloads all states from the top down. */
  pub fn clear(&mut self)
  {
    while self.states.len() > 0
    { self.states.pop().unload(); }
  }

  /** Updating and rendering. **/
  pub fn update(&mut self, delta: f32)
  {
    assert!(self.states.len() > 0);

    for x in self.states.mut_iter()
    {
      if x.update(delta)
      { break; }
    }
  }

  pub fn render(&mut self)
  {
    assert!(self.states.len() > 0);

    for x in self.states.mut_iter()
    {
      if x.render()
      { break; }
    }
  }

  /** Input handling. **/
  pub fn key_action(&mut self, key: i32, action: i32, mods: i32)
  {
    assert!(self.states.len() > 0);

    for x in self.states.mut_rev_iter()
    {
      if x.key_action(key, action, mods)
      { break; }
    }
  }

  pub fn key_char(&mut self, ch: char)
  {
    assert!(self.states.len() > 0);

    for x in self.states.mut_rev_iter()
    {
      if x.key_char(ch)
      { break; }
    }
  }

  pub fn mouse_action(&mut self, button: i32, action: i32, mods: i32)
  {
    assert!(self.states.len() > 0);

    for x in self.states.mut_rev_iter()
    {
      if x.mouse_action(button, action, mods)
      { break; }
    }
  }

  pub fn mouse_moved(&mut self, x: f32, y: f32)
  {
    assert!(self.states.len() > 0);

    for state in self.states.mut_rev_iter()
    {
      if state.mouse_moved(x, y)
      { break; }
    }
  }
}

