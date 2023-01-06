use self::{defender::Defender, diffuser::Diffuser, mine::Mine};

pub mod defender;
pub mod diffuser;
pub mod mine;

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

    //gets pos of all the attackers

    #[allow(dead_code)]
    pub fn simulate() {
        todo!()
    }
}
