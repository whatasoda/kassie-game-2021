mod _interfaces;
pub mod batting;
mod hit_ball;
pub mod pitching;

use std::cell::RefCell;
use std::cell::{Ref, RefMut};
use std::rc::Rc;

pub use _interfaces::*;

macro_rules! GameState {
    (
        $trait_name:ident,
        $state_trait_name:ident,
        $borrow:ident,
        $borrow_mut:ident
    ) => {
        pub trait $trait_name
        where
            Self::$state_trait_name: _interfaces::$state_trait_name,
        {
            type $state_trait_name;
            fn $borrow(&self) -> Ref<'_, Self::$state_trait_name>;
            fn $borrow_mut(&self) -> RefMut<'_, Self::$state_trait_name>;
        }
    };
}

GameState!(GameStateBatting, Batting, batting, batting_mut);
GameState!(GameStatePitching, Pitching, pitching, pitching_mut);
GameState!(GameStateHitBall, HitBall, hit_ball, hit_ball_mut);

pub struct BattingSceneGameState<B, P> {
    batting: Rc<RefCell<B>>,
    pitching: Rc<RefCell<P>>,
}

impl<B, P> BattingSceneGameState<B, P>
where
    B: Batting,
    P: Pitching,
{
    pub fn new(batting: Rc<RefCell<B>>, pitching: Rc<RefCell<P>>) -> Self {
        Self { batting, pitching }
    }
}

impl<B, P> GameStateBatting for BattingSceneGameState<B, P>
where
    B: Batting,
{
    type Batting = B;
    fn batting(&self) -> Ref<'_, Self::Batting> {
        self.batting.borrow()
    }
    fn batting_mut(&self) -> RefMut<'_, Self::Batting> {
        self.batting.borrow_mut()
    }
}

impl<B, P> GameStatePitching for BattingSceneGameState<B, P>
where
    P: Pitching,
{
    type Pitching = P;
    fn pitching(&self) -> Ref<'_, Self::Pitching> {
        self.pitching.borrow()
    }
    fn pitching_mut(&self) -> RefMut<'_, Self::Pitching> {
        self.pitching.borrow_mut()
    }
}
