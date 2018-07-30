use ir::Function; //Inst
use isa::TargetIsa;
use dominator_tree::DominatorTree;
use regalloc::live_value_tracker::LiveValueTracker;
use regalloc::liveness::Liveness;
use cursor::{Cursor, FuncCursor};
use std::collections::HashSet;
// use ir::instructions::{BranchInfo, CallInfo, InstructionData};
// use ir::builder::InstBuilder;
use ir::{self, InstBuilder};

// use ir::entities::Inst
// use regalloc::RegDiversions;

// emit_stackmaps() function prints the Instruction and the liveness of its defs and ops
// by traversing the dominator tree in a post order fashion
pub fn emit_stackmaps(isa: &TargetIsa, func: &mut Function, domtree: &mut DominatorTree,
    liveness: &mut Liveness, tracker: &mut LiveValueTracker) {

    // Visit EBBs in post-order
    // A `FuncCursor` holds a mutable reference to a whole `ir::Function` while keeping a position
    // too. The function can be re-borrowed by accessing the public `cur.func` member.
    let mut pos = FuncCursor::new(func);
    let mut ebbs_for_stackmap = HashSet::new();

    for &ebb in domtree.cfg_postorder().iter() {
        println!("Ebb Number is {:?}", ebb);
        // call ebb_top && drop_dead_params
        tracker.ebb_top(ebb, &pos.func.dfg, liveness, &pos.func.layout, domtree);
        tracker.drop_dead_params();

        // From the top of the ebb, step through the instructions
        pos.goto_top(ebb);

        while let Some(inst) = pos.next_inst() {
            // Get opcode of instruction
            let opcode = pos.func.dfg[inst].opcode();

            println!("Instruction Data: {}", pos.func.dfg.display_inst(inst, None));

            // Check if it's a branch instruction
            if opcode.is_branch() {
                // Find what the branch destination is
                let branch_dest = pos.func.dfg[inst].branch_destination();
                ebbs_for_stackmap.insert(branch_dest);

                println!("{}", opcode);
            }

            // Process the instruction
            tracker.process_inst(inst, &pos.func.dfg, liveness);

            // Get rid of values that have either (1) Dead Definitions or (2) Killed by Inst
            tracker.drop_dead(inst);

            // create an empty vector to store the live values in
            let mut live_value_list = Vec::new();

            // Grab the values that are still live
            let live_info = tracker.live();

            if live_info.len() != 0 {

                for value_in_list in live_info {
                    live_value_list.push(value_in_list.value);
                }
            }

            // live_value_list will have the list of live values in this instruction
            // print contents of array
            println!("In {:?}, {:?} has live values: ", ebb, inst);
            print!("   ");
            if live_value_list.len() == 0 {
                print!("no live values");
            }
            else {
                for val in live_value_list {
                    print!("{:?} ", val);
                }
            }

            println!();

        } // end while loop for instructions
    } // end for loop for ebb

    // loop through items in HashSet to insert stackmap instruction
    for ebb in ebbs_for_stackmap {
        pos.goto_first_insertion_point(ebb.unwrap());

        // insert stackmap instruction
        let mut value_list = Vec::new();
        pos.ins().stackmap(&value_list);

        // println!("Inserted Instruction Data: {}", pos.func.dfg.display_inst(pos.current_inst().unwrap(), None));

    }
}
