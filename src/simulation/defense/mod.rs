use self::{defender::Defender, diffuser::Diffuser, mine::Mine};

pub mod defender;
pub mod diffuser;
pub mod mine;

#[allow(dead_code)]
pub struct DefenseManager {
    pub defenders: Vec<Defender>,
    pub diffusers: Vec<Diffuser>,
    pub mine: Vec<Mine>,
}

impl DefenseManager {
    // #[allow(dead_code)]
    // pub fn new() -> Self {
    //     todo!()
    // }

    #[allow(dead_code)]
    pub fn simulate() {
        todo!()
    }
}
