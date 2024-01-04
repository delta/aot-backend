use super::util::NewAttack;
use crate::{constants::*, models::AttackerType};
use anyhow::Result;

use std::collections::{HashMap, HashSet};

pub fn is_attack_valid(
    new_attack: &NewAttack,
    valid_road_paths: HashSet<(i32, i32)>,
    valid_emp_ids: HashSet<i32>,
    no_of_bombs: &i32,
    no_of_attackers: &i32,
    attacker_types: &HashMap<i32, AttackerType>,
) -> Result<()> {
    if new_attack.attackers.is_empty()
        || new_attack.attackers.len() != new_attack.no_of_attackers as usize
        || new_attack.no_of_attackers > *no_of_attackers
    {
        return Err(anyhow::anyhow!("Invalid count of attackers"));
    }
    let mut total_attack_bomb_count = 0;
    for current_attacker in new_attack.attackers.iter() {
        //Check if its Not a valid Attacker Type
        if let Some(attacker_type) = attacker_types.get(&current_attacker.attacker_type) {
            let attacker_path = &current_attacker.attacker_path;
            let mut attack_bomb_count = 0;
            if attacker_path.is_empty() {
                return Err(anyhow::anyhow!("Attacker path cannot be empty"));
            }
            for i in 0..attacker_path.len() {
                let current_path = &attacker_path[i];
                if current_path.is_emp {
                    attack_bomb_count += 1;
                    if let (Some(emp_type), Some(emp_time)) =
                        (current_path.emp_type, current_path.emp_time)
                    {
                        // check if emp_id is valid
                        if !valid_emp_ids.contains(&emp_type) {
                            return Err(anyhow::anyhow!("Invalid Emp type"));
                        }
                        // check if emp_time is valid
                        let game_minutes = GAME_MINUTES_PER_FRAME
                            * (((i as f64) / (attacker_type.speed as f64)).ceil() as i32
                                + ATTACKER_RESTRICTED_FRAMES);
                        if emp_time < game_minutes {
                            return Err(anyhow::anyhow!("Invalid Emp Time"));
                        }
                    } else {
                        return Err(anyhow::anyhow!("Invalid Emp path"));
                    }
                }
                if !valid_road_paths.contains(&(current_path.x_coord, current_path.y_coord)) {
                    return Err(anyhow::anyhow!("Attacker can move only through Road"));
                }
                if i > 0 {
                    let previous_path = &attacker_path[i - 1];
                    let path_difference = (previous_path.x_coord - current_path.x_coord).abs()
                        + (previous_path.y_coord - current_path.y_coord).abs();
                    // attacker should move every frame
                    if path_difference != 1 {
                        return Err(anyhow::anyhow!("Invalid Attacker Path Sequence"));
                    }
                }
            }

            //check Max_no_of_bombs For individual attackers
            if attack_bomb_count > attacker_type.amt_of_emps {
                return Err(anyhow::anyhow!("Amount Of Emp xxceeds for an attacker"));
            }
            total_attack_bomb_count += attack_bomb_count;
        } else {
            return Err(anyhow::anyhow!("Invalid Attacker Type"));
        }
    }

    if total_attack_bomb_count > *no_of_bombs {
        return Err(anyhow::anyhow!("Total Amount Of emps used exceeds"));
    }

    Ok(())
}
